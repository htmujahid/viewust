use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::model::{DirectoryUsage, UsageEntry};
use crate::error::{AppError, Result};

/// How long a measured folder is trusted before it is measured again.
const FRESH: Duration = Duration::from_secs(600);
/// Measuring something enormous stops here and says the answer is incomplete.
const DEADLINE: Duration = Duration::from_secs(90);
/// Folders this many levels below the one being opened are remembered, so opening them is instant.
const REMEMBER_DEPTH: usize = 3;
/// The biggest entries sent for one folder; the rest are summed into one line.
const KEEP: usize = 500;
const MAX_REMEMBERED: usize = 300_000;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
struct Measured {
    size: u64,
    files: u64,
    unreadable: bool,
}

/// What a file takes on disk (whole blocks), not its length: this is what `du` reports.
fn on_disk(md: &Metadata) -> u64 {
    md.blocks() * 512
}

#[derive(Clone, Default)]
pub struct UsageService(Arc<Mutex<HashMap<PathBuf, (Instant, Measured)>>>);

impl UsageService {
    fn lookup(&self, path: &Path) -> Option<Measured> {
        let map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        map.get(path)
            .filter(|(at, _)| at.elapsed() < FRESH)
            .map(|(_, m)| *m)
    }

    fn remember(&self, items: impl IntoIterator<Item = (PathBuf, Measured)>) {
        let mut map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() > MAX_REMEMBERED {
            map.clear();
        }
        let now = Instant::now();
        map.extend(items.into_iter().map(|(p, m)| (p, (now, m))));
    }

    fn forget_under(&self, path: &Path) {
        let mut map = self.0.lock().unwrap_or_else(|e| e.into_inner());
        map.retain(|p, _| !p.starts_with(path));
    }
}

/// One folder found during a pass. Folders are numbered in the order they are found, so a
/// parent always has a lower number than anything inside it.
struct Node {
    parent: Option<usize>,
    depth: usize,
    path: Option<PathBuf>,
    total: Measured,
}

struct Queue {
    waiting: VecDeque<(usize, PathBuf)>,
    /// Folders waiting or being read right now; when it reaches zero the pass is over.
    open: usize,
}

/// One measuring pass over a device: never leaves it, never follows a link. Every worker pulls
/// folders from a shared queue, so one enormous folder is read by all of them at once.
struct Walk {
    dev: u64,
    deadline: Instant,
    incomplete: AtomicBool,
    visited: AtomicUsize,
    links: Mutex<HashSet<(u64, u64)>>,
    nodes: Mutex<Vec<Node>>,
    queue: Mutex<Queue>,
    wake: Condvar,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Walk {
    fn new(dev: u64) -> Self {
        Self {
            dev,
            deadline: Instant::now() + DEADLINE,
            incomplete: AtomicBool::new(false),
            visited: AtomicUsize::new(0),
            links: Mutex::default(),
            nodes: Mutex::default(),
            queue: Mutex::new(Queue {
                waiting: VecDeque::new(),
                open: 0,
            }),
            wake: Condvar::new(),
        }
    }

    fn out_of_time(&self) -> bool {
        if self.incomplete.load(Ordering::Relaxed) {
            return true;
        }
        if self.visited.fetch_add(1, Ordering::Relaxed) % 2048 == 0
            && Instant::now() > self.deadline
        {
            self.incomplete.store(true, Ordering::Relaxed);
        }
        self.incomplete.load(Ordering::Relaxed)
    }

    /// A file with several names takes space once, however many folders hold it.
    fn first_sight(&self, md: &Metadata) -> bool {
        md.nlink() <= 1 || lock(&self.links).insert((md.dev(), md.ino()))
    }

    fn add_root(&self, path: PathBuf) -> usize {
        let mut nodes = lock(&self.nodes);
        nodes.push(Node {
            parent: None,
            depth: 0,
            path: Some(path.clone()),
            total: Measured::default(),
        });
        let id = nodes.len() - 1;
        drop(nodes);
        let mut q = lock(&self.queue);
        q.waiting.push_back((id, path));
        q.open += 1;
        id
    }

    /// Reads one folder: its files are summed on the spot, its subfolders are queued.
    fn read(&self, id: usize, dir: &Path) {
        let mut own = Measured::default();
        if let Ok(md) = fs::symlink_metadata(dir) {
            own.size = on_disk(&md);
        }
        let mut subfolders = Vec::new();
        match fs::read_dir(dir) {
            Err(_) => own.unreadable = true,
            Ok(entries) => {
                for entry in entries {
                    if self.out_of_time() {
                        break;
                    }
                    let Ok(entry) = entry else {
                        own.unreadable = true;
                        continue;
                    };
                    let Ok(md) = entry.metadata() else {
                        own.unreadable = true;
                        continue;
                    };
                    if md.is_dir() {
                        if md.dev() == self.dev {
                            subfolders.push(entry.path());
                        }
                    } else if self.first_sight(&md) {
                        own.size += on_disk(&md);
                        own.files += 1;
                    }
                }
            }
        }

        let mut nodes = lock(&self.nodes);
        let depth = nodes[id].depth + 1;
        nodes[id].total = own;
        let first = nodes.len();
        for path in &subfolders {
            nodes.push(Node {
                parent: Some(id),
                depth,
                path: (depth <= REMEMBER_DEPTH).then(|| path.clone()),
                total: Measured::default(),
            });
        }
        drop(nodes);
        if !subfolders.is_empty() {
            let mut q = lock(&self.queue);
            q.open += subfolders.len();
            q.waiting.extend(
                subfolders
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| (first + i, p)),
            );
            self.wake.notify_all();
        }
    }

    fn work(&self) {
        loop {
            let job = {
                let mut q = lock(&self.queue);
                loop {
                    if let Some(job) = q.waiting.pop_front() {
                        break Some(job);
                    }
                    if q.open == 0 {
                        break None;
                    }
                    q = self.wake.wait(q).unwrap_or_else(|e| e.into_inner());
                }
            };
            let Some((id, dir)) = job else {
                self.wake.notify_all();
                return;
            };
            if !self.incomplete.load(Ordering::Relaxed) {
                self.read(id, &dir);
            }
            let mut q = lock(&self.queue);
            q.open -= 1;
            if q.open == 0 {
                self.wake.notify_all();
            }
        }
    }

    /// Measures every folder in `roots`, returning each one's total and noting the
    /// first few levels beneath them for later.
    fn measure(&self, roots: &[PathBuf]) -> (Vec<Measured>, Vec<(PathBuf, Measured)>) {
        let ids: Vec<usize> = roots.iter().map(|p| self.add_root(p.clone())).collect();
        let workers = std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .clamp(2, 8);
        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| self.work());
            }
        });

        let mut nodes = lock(&self.nodes);
        for i in (0..nodes.len()).rev() {
            if let Some(parent) = nodes[i].parent {
                let child = nodes[i].total;
                let up = &mut nodes[parent].total;
                up.size += child.size;
                up.files += child.files;
                up.unreadable |= child.unreadable;
            }
        }
        let found = nodes
            .iter()
            .filter_map(|n| n.path.clone().map(|p| (p, n.total)))
            .collect();
        (ids.iter().map(|&i| nodes[i].total).collect(), found)
    }
}

struct Child {
    name: String,
    path: PathBuf,
    kind: &'static str,
    size: u64,
    measured: Option<Measured>,
    mount: bool,
}

fn kind_of(md: &Metadata) -> &'static str {
    if md.is_dir() {
        "dir"
    } else if md.file_type().is_symlink() {
        "link"
    } else if md.is_file() {
        "file"
    } else {
        "other"
    }
}

pub(crate) fn directory(
    service: &UsageService,
    path: &str,
    refresh: bool,
) -> Result<DirectoryUsage> {
    let started = Instant::now();
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(AppError::Other(format!(
            "{} is not a full path",
            path.display()
        )));
    }
    let path = fs::canonicalize(path)
        .map_err(|e| AppError::Other(format!("Can't open {}: {e}", path.display())))?;
    let md = fs::metadata(&path)
        .map_err(|e| AppError::Other(format!("Can't open {}: {e}", path.display())))?;
    if !md.is_dir() {
        return Err(AppError::Other(format!(
            "{} is not a folder",
            path.display()
        )));
    }
    if refresh {
        service.forget_under(&path);
    }

    let mut unreadable = false;
    let mut children = Vec::new();
    match fs::read_dir(&path) {
        Err(_) => unreadable = true,
        Ok(entries) => {
            for entry in entries {
                let Ok(entry) = entry else {
                    unreadable = true;
                    continue;
                };
                let Ok(cmd) = entry.metadata() else {
                    unreadable = true;
                    continue;
                };
                let kind = kind_of(&cmd);
                let mount = cmd.is_dir() && cmd.dev() != md.dev();
                children.push(Child {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    path: entry.path(),
                    kind,
                    size: if cmd.is_dir() { 0 } else { on_disk(&cmd) },
                    measured: None,
                    mount,
                });
            }
        }
    }

    let walk = Walk::new(md.dev());
    let mut todo = Vec::new();
    for child in children.iter_mut().filter(|c| c.kind == "dir" && !c.mount) {
        match service.lookup(&child.path) {
            Some(m) => child.measured = Some(m),
            None => todo.push(child.path.clone()),
        }
    }
    let (measured, found) = walk.measure(&todo);
    let complete = !walk.incomplete.load(Ordering::Relaxed);
    if complete {
        service.remember(found);
    }
    let mut fresh = todo.iter().zip(measured).collect::<HashMap<_, _>>();
    for child in children.iter_mut().filter(|c| c.kind == "dir" && !c.mount) {
        if child.measured.is_none() {
            child.measured = fresh.remove(&child.path);
        }
    }

    let mut entries: Vec<UsageEntry> = children
        .into_iter()
        .map(|c| {
            let m = c.measured.unwrap_or_default();
            unreadable |= m.unreadable;
            UsageEntry {
                name: c.name,
                path: c.path.to_string_lossy().into_owned(),
                kind: c.kind,
                size: if c.kind == "dir" { m.size } else { c.size },
                files: m.files,
                unreadable: m.unreadable,
                mount: c.mount,
            }
        })
        .collect();
    entries.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));

    let hidden = entries.split_off(entries.len().min(KEEP));
    let hidden_size = hidden.iter().map(|e| e.size).sum::<u64>();
    let total = on_disk(&md) + hidden_size + entries.iter().map(|e| e.size).sum::<u64>();
    Ok(DirectoryUsage {
        path: path.to_string_lossy().into_owned(),
        total,
        entries,
        hidden_count: hidden.len(),
        hidden_size,
        unreadable,
        incomplete: !complete,
        took_ms: started.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("viewust-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn file(&self, rel: &str, bytes: usize) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::File::create(p)
                .unwrap()
                .write_all(&vec![7u8; bytes])
                .unwrap();
        }
        fn open(&self, path: &Path) -> DirectoryUsage {
            directory(&UsageService::default(), path.to_str().unwrap(), false).unwrap()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    use std::os::unix::fs::PermissionsExt;

    fn find<'a>(u: &'a DirectoryUsage, name: &str) -> &'a UsageEntry {
        u.entries.iter().find(|e| e.name == name).unwrap()
    }

    #[test]
    fn folders_add_up_what_is_inside_them_and_sort_biggest_first() {
        let s = Scratch::new("sum");
        s.file("big/a.bin", 400_000);
        s.file("big/deep/b.bin", 300_000);
        s.file("small/c.bin", 10_000);
        s.file("loose.bin", 50_000);
        let u = s.open(&s.0);
        let names: Vec<&str> = u.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["big", "loose.bin", "small"]);
        let big = find(&u, "big");
        assert!(big.size >= 700_000 && big.size < 760_000, "{}", big.size);
        assert_eq!(big.files, 2);
        assert_eq!(find(&u, "loose.bin").kind, "file");
        assert!(u.total >= big.size + find(&u, "small").size);
        assert!(!u.incomplete && !u.unreadable);
    }

    #[test]
    fn a_child_folder_can_be_opened_and_matches_its_parents_figure() {
        let s = Scratch::new("child");
        s.file("a/b/x.bin", 200_000);
        s.file("a/y.bin", 100_000);
        let top = s.open(&s.0);
        let a = find(&top, "a");
        let inside = s.open(Path::new(&a.path));
        assert_eq!(inside.total, a.size);
        assert_eq!(find(&inside, "b").files, 1);
    }

    #[test]
    fn a_file_with_two_names_counts_once_and_links_are_not_followed() {
        let s = Scratch::new("links");
        s.file("one/data.bin", 300_000);
        fs::create_dir_all(s.0.join("two")).unwrap();
        fs::hard_link(s.0.join("one/data.bin"), s.0.join("two/again.bin")).unwrap();
        std::os::unix::fs::symlink("/usr", s.0.join("shortcut")).unwrap();
        let u = s.open(&s.0);
        let (one, two) = (find(&u, "one").size, find(&u, "two").size);
        assert!(one + two < 400_000, "counted twice: {one} + {two}");
        assert_eq!(find(&u, "shortcut").kind, "link");
        assert!(find(&u, "shortcut").size < 4096);
    }

    #[test]
    fn an_unreadable_folder_is_flagged_instead_of_failing() {
        if std::fs::read_dir("/root").is_ok() {
            return; // running as root: nothing is unreadable
        }
        let s = Scratch::new("denied");
        s.file("locked/secret.bin", 100_000);
        s.file("open/ok.bin", 100_000);
        fs::set_permissions(s.0.join("locked"), fs::Permissions::from_mode(0o000)).unwrap();
        let u = s.open(&s.0);
        assert!(find(&u, "locked").unreadable);
        assert!(!find(&u, "open").unreadable);
        assert!(u.unreadable);
        fs::set_permissions(s.0.join("locked"), fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn measured_folders_are_remembered_until_a_refresh() {
        let s = Scratch::new("cache");
        s.file("d/x.bin", 100_000);
        let service = UsageService::default();
        let path = s.0.to_str().unwrap();
        let first = directory(&service, path, false).unwrap();
        s.file("d/more.bin", 500_000);
        let cached = directory(&service, path, false).unwrap();
        assert_eq!(find(&cached, "d").size, find(&first, "d").size);
        let fresh = directory(&service, path, true).unwrap();
        assert!(find(&fresh, "d").size > find(&first, "d").size + 400_000);
    }

    #[test]
    fn only_the_biggest_entries_are_listed_and_the_rest_are_summed() {
        let s = Scratch::new("many");
        for i in 0..(KEEP + 20) {
            s.file(&format!("f{i:04}.bin"), 1);
        }
        let u = s.open(&s.0);
        assert_eq!(u.entries.len(), KEEP);
        assert_eq!(u.hidden_count, 20);
        assert!(u.hidden_size > 0);
    }

    #[test]
    fn bad_paths_are_explained() {
        let service = UsageService::default();
        assert!(directory(&service, "relative/path", false).is_err());
        assert!(directory(&service, "/definitely/not/here", false).is_err());
        assert!(directory(&service, "/etc/hostname", false)
            .unwrap_err()
            .to_string()
            .contains("not a folder"));
    }
}
