use std::path::Path;

use super::model::{ContainerRow, Containers, ImageRow};
use crate::common::cmd::run;

/// Lines of `docker ps -a --format "{{.Names}}\t{{.Image}}\t{{.State}}\t{{.Status}}"`.
pub(crate) fn parse_containers(text: &str) -> Vec<ContainerRow> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            Some(ContainerRow {
                name: f.next()?.to_owned(),
                image: f.next()?.to_owned(),
                state: f.next()?.to_owned(),
                status: f.next().unwrap_or("").to_owned(),
            })
        })
        .collect()
}

/// Lines of `docker images --format "{{.Repository}}:{{.Tag}}\t{{.Size}}"`.
pub(crate) fn parse_images(text: &str) -> Vec<ImageRow> {
    text.lines()
        .filter_map(|line| {
            let (name, size) = line.split_once('\t')?;
            Some(ImageRow {
                name: name.to_owned(),
                size: size.trim().to_owned(),
            })
        })
        .collect()
}

fn find_runtime() -> Option<&'static str> {
    ["docker", "podman"].into_iter().find(|bin| {
        ["/usr/bin", "/usr/local/bin", "/bin"]
            .iter()
            .any(|d| Path::new(&format!("{d}/{bin}")).exists())
    })
}

pub(crate) fn snapshot() -> Containers {
    let Some(runtime) = find_runtime() else {
        return Containers {
            runtime: None,
            note: Some("Neither docker nor podman is installed.".into()),
            running: 0,
            containers: Vec::new(),
            images: Vec::new(),
        };
    };
    let listed = run(
        runtime,
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}}\t{{.Image}}\t{{.State}}\t{{.Status}}",
        ],
    );
    let Some(listed) = listed else {
        return Containers {
            runtime: Some(runtime),
            note: Some(format!(
                "{runtime} is installed but didn't answer: its service may be stopped, or this \
                 user may not be in its group."
            )),
            running: 0,
            containers: Vec::new(),
            images: Vec::new(),
        };
    };
    let mut containers = parse_containers(&listed);
    containers.sort_by(|a, b| {
        (a.state != "running")
            .cmp(&(b.state != "running"))
            .then_with(|| a.name.cmp(&b.name))
    });
    containers.truncate(200);
    let images = run(
        runtime,
        &["images", "--format", "{{.Repository}}:{{.Tag}}\t{{.Size}}"],
    )
    .map(|t| {
        let mut rows = parse_images(&t);
        rows.truncate(200);
        rows
    })
    .unwrap_or_default();
    Containers {
        runtime: Some(runtime),
        note: None,
        running: containers.iter().filter(|c| c.state == "running").count(),
        containers,
        images,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_lines_split_on_tabs() {
        let text = "web\tnginx:1.27\trunning\tUp 2 hours\ndb\tpostgres:16\texited\tExited (0) 3 days ago\n";
        let c = parse_containers(text);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "web");
        assert_eq!(c[1].state, "exited");
        assert_eq!(c[1].status, "Exited (0) 3 days ago");
    }

    #[test]
    fn image_lines_give_name_and_size() {
        let i = parse_images("nginx:1.27\t67.7MB\npostgres:16\t432MB\n");
        assert_eq!(i.len(), 2);
        assert_eq!(i[1].size, "432MB");
    }

    #[test]
    fn this_machine_answers_honestly_about_containers() {
        // Whatever is installed, the answer must say what it found.
        let c = snapshot();
        if c.runtime.is_none() {
            assert!(c.note.is_some());
        }
    }
}
