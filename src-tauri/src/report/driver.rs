//! Kernel driver and module facts.

use crate::common::sysfs::*;
use crate::common::Details;
use std::process::Command;

fn modinfo(module: &str, field: &str) -> Vec<String> {
    Command::new("modinfo")
        .args(["-F", field, module])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_owned())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn driver_rows(d: &mut Details, section: &str, driver: &str) {
    d.add(section, "Driver", driver);
    let description = modinfo(driver, "description");
    if description.is_empty() {
        d.add(
            section,
            "Module",
            "Built into the kernel (or not a loadable module)",
        );
        return;
    }
    d.add(section, "Description", description.join(" "));
    let authors = modinfo(driver, "author");
    if !authors.is_empty() {
        d.add(section, "Author", authors.join(", "));
    }
    d.add_opt(
        section,
        "License",
        modinfo(driver, "license").into_iter().next(),
    );
    d.add_opt(
        section,
        "Version",
        modinfo(driver, "version").into_iter().next(),
    );
    d.add_opt(
        section,
        "Module file",
        modinfo(driver, "filename").into_iter().next(),
    );
    let depends = modinfo(driver, "depends");
    if !depends.is_empty() && !depends.join("").is_empty() {
        d.add(section, "Depends on", depends.join(", "));
    }
    let used = read(format!("/sys/module/{}/refcnt", driver.replace('-', "_")));
    d.add_opt(section, "Users", used.map(|n| format!("{n} reference(s)")));
}
