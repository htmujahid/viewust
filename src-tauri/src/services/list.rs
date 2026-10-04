use std::collections::HashMap;

use super::model::{Overview, ServiceRow, Snapshot};
use crate::common::cmd::run;

const SYSTEMCTL: &str = "systemctl";

pub(crate) struct Unit {
    pub(crate) unit: String,
    pub(crate) load: String,
    pub(crate) active: String,
    pub(crate) sub: String,
    pub(crate) description: String,
}

pub(crate) fn parse_units(text: &str) -> Vec<Unit> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let unit = words.next()?.to_owned();
            if !unit.ends_with(".service") {
                return None;
            }
            let load = words.next()?.to_owned();
            let active = words.next()?.to_owned();
            let sub = words.next()?.to_owned();
            let description = words.collect::<Vec<_>>().join(" ");
            Some(Unit {
                unit,
                load,
                active,
                sub,
                description,
            })
        })
        .collect()
}

pub(crate) fn parse_unit_files(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let unit = words.next()?;
            let state = words.next()?;
            unit.ends_with(".service")
                .then(|| (unit.to_owned(), state.to_owned()))
        })
        .collect()
}

pub fn parse_properties(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

pub(crate) fn parse_blocks(text: &str) -> Vec<HashMap<String, String>> {
    text.split("\n\n")
        .map(parse_properties)
        .filter(|b| !b.is_empty())
        .collect()
}

pub(crate) fn counter(value: Option<&String>) -> Option<u64> {
    value
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v != u64::MAX)
}

pub(crate) fn overview(rows: &[ServiceRow]) -> Overview {
    Overview {
        total: rows.len(),
        running: rows.iter().filter(|r| r.sub == "running").count(),
        exited: rows.iter().filter(|r| r.sub == "exited").count(),
        failed: rows.iter().filter(|r| r.active == "failed").count(),
        inactive: rows.iter().filter(|r| r.active == "inactive").count(),
        enabled: rows
            .iter()
            .filter(|r| r.enabled.starts_with("enabled"))
            .count(),
        memory: rows.iter().filter_map(|r| r.memory).sum(),
    }
}

pub(crate) fn build(
    units: Vec<Unit>,
    files: &HashMap<String, String>,
    runtime: &HashMap<String, (Option<u32>, Option<u64>)>,
) -> Vec<ServiceRow> {
    let mut rows: Vec<ServiceRow> = units
        .into_iter()
        .filter(|u| u.load != "not-found")
        .map(|u| {
            row(
                &u.unit,
                &u.load,
                &u.active,
                &u.sub,
                &u.description,
                files,
                runtime,
            )
        })
        .collect();
    let known: Vec<String> = rows.iter().map(|r| r.unit.clone()).collect();
    for unit in files.keys() {
        if unit.contains("@.") || known.contains(unit) {
            continue;
        }
        rows.push(row(
            unit, "unloaded", "inactive", "dead", "", files, runtime,
        ));
    }
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

fn row(
    unit: &str,
    load: &str,
    active: &str,
    sub: &str,
    description: &str,
    files: &HashMap<String, String>,
    runtime: &HashMap<String, (Option<u32>, Option<u64>)>,
) -> ServiceRow {
    let (main_pid, memory) = runtime.get(unit).copied().unwrap_or((None, None));
    ServiceRow {
        unit: unit.to_owned(),
        name: unit.trim_end_matches(".service").to_owned(),
        description: description.to_owned(),
        load: load.to_owned(),
        active: active.to_owned(),
        sub: sub.to_owned(),
        enabled: files.get(unit).cloned().unwrap_or_else(|| "-".into()),
        main_pid,
        memory,
    }
}

pub(crate) fn snapshot() -> Snapshot {
    let Some(units) = run(
        SYSTEMCTL,
        &[
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--plain",
            "--no-pager",
        ],
    ) else {
        return Snapshot {
            available: false,
            overview: Overview::default(),
            services: Vec::new(),
        };
    };
    let files = run(
        SYSTEMCTL,
        &[
            "list-unit-files",
            "--type=service",
            "--no-legend",
            "--no-pager",
        ],
    )
    .map(|t| parse_unit_files(&t))
    .unwrap_or_default();
    let runtime: HashMap<String, (Option<u32>, Option<u64>)> = run(
        SYSTEMCTL,
        &[
            "show",
            "*.service",
            "--property=Id,MainPID,MemoryCurrent",
            "--no-pager",
        ],
    )
    .map(|t| {
        parse_blocks(&t)
            .into_iter()
            .filter_map(|b| {
                let id = b.get("Id")?.clone();
                let pid = counter(b.get("MainPID"))
                    .filter(|p| *p > 0)
                    .map(|p| p as u32);
                Some((id, (pid, counter(b.get("MemoryCurrent")))))
            })
            .collect()
    })
    .unwrap_or_default();

    let services = build(parse_units(&units), &files, &runtime);
    Snapshot {
        available: true,
        overview: overview(&services),
        services,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNITS: &str = "\
accounts-daemon.service loaded active   running Accounts Service
alsa-restore.service    loaded active   exited  Save/Restore Sound Card State
broken.service          loaded failed   failed  A Broken Thing
ghost.service           not-found inactive dead ghost.service
bluetooth.target        loaded active   active  Not a service
";
    const FILES: &str = "\
accounts-daemon.service enabled enabled
alsa-card-wait@.service static  -
cups.service            disabled enabled
";

    #[test]
    fn units_are_split_into_columns_with_the_description_kept_whole() {
        let units = parse_units(UNITS);
        assert_eq!(units.len(), 4);
        assert_eq!(units[0].unit, "accounts-daemon.service");
        assert_eq!(units[0].sub, "running");
        assert_eq!(units[1].description, "Save/Restore Sound Card State");
    }

    #[test]
    fn unit_files_map_to_their_boot_state() {
        let files = parse_unit_files(FILES);
        assert_eq!(files["cups.service"], "disabled");
        assert_eq!(files["accounts-daemon.service"], "enabled");
    }

    #[test]
    fn blocks_are_separated_by_blank_lines() {
        let blocks = parse_blocks("Id=a.service\nMainPID=7\n\nId=b.service\nMainPID=0\n\n");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[1]["Id"], "b.service");
    }

    #[test]
    fn unset_counters_are_missing_not_huge() {
        let max = u64::MAX.to_string();
        assert_eq!(counter(Some(&max)), None);
        assert_eq!(counter(Some(&"[not set]".to_string())), None);
        assert_eq!(counter(Some(&"4096".to_string())), Some(4096));
        assert_eq!(counter(None), None);
    }

    #[test]
    fn rows_drop_ghosts_add_unloaded_files_and_skip_templates() {
        let files = parse_unit_files(FILES);
        let mut runtime = HashMap::new();
        runtime.insert(
            "accounts-daemon.service".to_owned(),
            (Some(900), Some(2048)),
        );
        let rows = build(parse_units(UNITS), &files, &runtime);
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["accounts-daemon", "alsa-restore", "broken", "cups"]);
        assert_eq!(rows[0].main_pid, Some(900));
        assert_eq!(rows[0].memory, Some(2048));
        assert_eq!(rows[3].load, "unloaded");
        assert_eq!(rows[3].enabled, "disabled");
        assert_eq!(rows[2].enabled, "-");
    }

    #[test]
    fn the_overview_counts_each_state() {
        let files = parse_unit_files(FILES);
        let rows = build(parse_units(UNITS), &files, &HashMap::new());
        let o = overview(&rows);
        assert_eq!((o.total, o.running, o.exited, o.failed), (4, 1, 1, 1));
        assert_eq!(o.enabled, 1);
        assert_eq!(o.inactive, 1);
    }
}
