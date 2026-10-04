use std::collections::HashSet;

use super::model::{KernelModule, ModuleInfo};
use crate::common::cmd::run;
use crate::common::sysfs::read;
use crate::common::Details;

/// `/proc/modules`: `name size refcount deps state address`, deps comma-separated or `-`.
pub(crate) fn parse_proc_modules(text: &str) -> Vec<KernelModule> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?.to_owned();
            let size = words.next()?.parse().ok()?;
            let _refcount = words.next()?;
            let used_by = words
                .next()?
                .split(',')
                .filter(|m| !m.is_empty() && *m != "-")
                .map(str::to_owned)
                .collect();
            Some(KernelModule {
                name,
                size,
                used_by,
            })
        })
        .collect()
}

pub(crate) fn list() -> Vec<KernelModule> {
    let mut modules = std::fs::read_to_string("/proc/modules")
        .map(|t| parse_proc_modules(&t))
        .unwrap_or_default();
    modules.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    modules
}

/// `modinfo` prints `key:   value`; a key can repeat (`parm`, `alias`).
pub(crate) fn parse_modinfo(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

/// The name goes to a program, so only what a module can really be called gets through.
pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(crate) fn info(name: &str) -> ModuleInfo {
    let missing = || ModuleInfo {
        name: name.to_owned(),
        found: false,
        details: Vec::new(),
    };
    if !valid_name(name) {
        return missing();
    }
    let loaded = list().into_iter().find(|m| m.name == name);
    let fields = run("modinfo", &["--", name])
        .map(|t| parse_modinfo(&t))
        .unwrap_or_default();
    if fields.is_empty() && loaded.is_none() {
        return missing();
    }
    let first = |key: &str| {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };

    let mut d = Details::new();
    d.add("Module", "Name", name);
    d.add_opt("Module", "Description", first("description"));
    d.add_opt("Module", "Author", first("author"));
    d.add_opt("Module", "License", first("license"));
    d.add_opt("Module", "Version", first("version"));
    d.add_opt("Module", "File", first("filename"));
    d.add_opt("Module", "Built for", first("vermagic"));
    if let Some(m) = &loaded {
        d.add(
            "Loaded",
            "Memory",
            crate::common::format::format_bytes(m.size),
        );
        d.add(
            "Loaded",
            "Used by",
            if m.used_by.is_empty() {
                "Nothing else".to_owned()
            } else {
                m.used_by.join(", ")
            },
        );
    } else {
        d.add("Loaded", "State", "Not loaded");
    }
    d.add_opt(
        "Needs",
        "Depends on",
        first("depends").filter(|v| !v.is_empty()),
    );
    d.add_opt("Needs", "Firmware", first("firmware"));
    d.add_opt("Signature", "Signed by", first("signer"));

    let mut seen = HashSet::new();
    for (key, value) in fields.iter().filter(|(k, _)| k == "parm").take(40) {
        let (parm, text) = value.split_once(':').unwrap_or((value.as_str(), ""));
        if !seen.insert(parm.to_owned()) {
            continue;
        }
        let current = read(format!("/sys/module/{name}/parameters/{parm}"));
        let _ = key;
        d.add(
            "Settings",
            parm,
            match (current, text.is_empty()) {
                (Some(v), false) => format!("{v} · {text}"),
                (Some(v), true) => v,
                (None, _) => text.to_owned(),
            },
        );
    }
    ModuleInfo {
        name: name.to_owned(),
        found: true,
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROC: &str = "\
nvidia_drm 94208 18 - Live 0x0000000000000000 (POE)
nvidia_modeset 1572864 11 nvidia_drm, Live 0xffffffffc0000000 (POE)
snd_hda_intel 61440 3 snd_hda_codec,snd_hda_core, Live 0x0
bad line
";

    #[test]
    fn loaded_modules_give_their_size_and_who_uses_them() {
        let m = parse_proc_modules(PROC);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].name, "nvidia_drm");
        assert!(m[0].used_by.is_empty());
        assert_eq!(m[1].used_by, ["nvidia_drm"]);
        assert_eq!(m[2].used_by, ["snd_hda_codec", "snd_hda_core"]);
        assert_eq!(m[1].size, 1_572_864);
    }

    #[test]
    fn modinfo_keeps_repeated_keys() {
        let f = parse_modinfo(
            "filename:   /lib/x.ko\nparm:   debug:Enable it (bool)\nparm:  mode:Pick (int)\n",
        );
        assert_eq!(f[0], ("filename".into(), "/lib/x.ko".into()));
        assert_eq!(f.iter().filter(|(k, _)| k == "parm").count(), 2);
    }

    #[test]
    fn only_real_module_names_reach_the_program() {
        assert!(valid_name("snd_hda_intel"));
        assert!(valid_name("i2c-core"));
        assert!(!valid_name(""));
        assert!(!valid_name("-a"));
        assert!(!valid_name("a b"));
        assert!(!valid_name("../x"));
        assert!(!valid_name("a;b"));
        assert!(!info("a;b").found);
    }

    #[test]
    fn an_unknown_module_is_reported_missing() {
        assert!(!info("definitely_not_a_module").found);
    }
}
