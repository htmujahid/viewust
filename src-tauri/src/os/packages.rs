use super::model::{PackageRow, Packages};
use crate::common::cmd::run;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Manager {
    Dpkg,
    Rpm,
    Pacman,
}

impl Manager {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Manager::Dpkg => "apt (dpkg)",
            Manager::Rpm => "rpm",
            Manager::Pacman => "pacman",
        }
    }

    fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Manager::Dpkg => (
                "dpkg-query",
                &[
                    "-W",
                    "-f=${db:Status-Abbrev}\t${Package}\t${Version}\t${Architecture}\n",
                ],
            ),
            Manager::Rpm => (
                "rpm",
                &[
                    "-qa",
                    "--qf",
                    "ok\t%{NAME}\t%{VERSION}-%{RELEASE}\t%{ARCH}\n",
                ],
            ),
            Manager::Pacman => ("pacman", &["-Q"]),
        }
    }
}

pub(crate) fn detect() -> Option<Manager> {
    let has = |p: &str| {
        ["/usr/bin", "/bin", "/usr/sbin"]
            .iter()
            .any(|d| std::path::Path::new(&format!("{d}/{p}")).exists())
    };
    [
        (Manager::Dpkg, "dpkg-query"),
        (Manager::Rpm, "rpm"),
        (Manager::Pacman, "pacman"),
    ]
    .into_iter()
    .find(|(_, tool)| has(tool))
    .map(|(m, _)| m)
}

/// Tab-separated `status name version arch`; dpkg lists removed-but-configured packages too,
/// which are not installed (their status isn't `ii`).
pub(crate) fn parse_tabbed(text: &str) -> Vec<PackageRow> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let status = f.next()?.trim();
            if !(status == "ii" || status == "ok") {
                return None;
            }
            Some(PackageRow {
                name: f.next()?.to_owned(),
                version: f.next()?.to_owned(),
                arch: f.next().map(str::to_owned).filter(|a| !a.is_empty()),
            })
        })
        .collect()
}

pub(crate) fn parse_pacman(text: &str) -> Vec<PackageRow> {
    text.lines()
        .filter_map(|line| {
            let (name, version) = line.split_once(' ')?;
            Some(PackageRow {
                name: name.to_owned(),
                version: version.trim().to_owned(),
                arch: None,
            })
        })
        .collect()
}

pub(crate) fn parse(manager: Manager, text: &str) -> Vec<PackageRow> {
    let mut rows = match manager {
        Manager::Pacman => parse_pacman(text),
        _ => parse_tabbed(text),
    };
    rows.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.arch.cmp(&b.arch)));
    rows
}

pub(crate) fn list() -> Packages {
    let Some(manager) = detect() else {
        return Packages {
            manager: None,
            packages: Vec::new(),
        };
    };
    let (program, args) = manager.command();
    Packages {
        manager: Some(manager.label()),
        packages: run(program, args)
            .map(|t| parse(manager, &t))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DPKG: &str = "\
ii \tbash\t5.2.21-2ubuntu4\tamd64
ii \tlibc6\t2.39-0ubuntu8\tamd64
ii \tlibc6\t2.39-0ubuntu8\ti386
rc \toldthing\t1.0\tamd64
un \tnothing\t\t
";

    #[test]
    fn only_installed_dpkg_packages_are_kept_and_sorted() {
        let p = parse(Manager::Dpkg, DPKG);
        let names: Vec<(&str, Option<&str>)> = p
            .iter()
            .map(|r| (r.name.as_str(), r.arch.as_deref()))
            .collect();
        assert_eq!(
            names,
            [
                ("bash", Some("amd64")),
                ("libc6", Some("amd64")),
                ("libc6", Some("i386"))
            ]
        );
        assert_eq!(p[0].version, "5.2.21-2ubuntu4");
    }

    #[test]
    fn rpm_lines_are_read_the_same_way() {
        let p = parse(
            Manager::Rpm,
            "ok\tbash\t5.2-1.fc40\tx86_64\nok\tzlib\t1.3-2\tx86_64\n",
        );
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].name, "zlib");
    }

    #[test]
    fn pacman_has_no_architecture_column() {
        let p = parse(Manager::Pacman, "bash 5.2.026-2\nlinux 6.9.1.arch1-1\n");
        assert_eq!(
            (p[0].name.as_str(), p[0].version.as_str(), p[0].arch.clone()),
            ("bash", "5.2.026-2", None)
        );
        assert_eq!(p.len(), 2);
    }
}
