use std::collections::HashMap;

use sysinfo::{Groups, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

use super::model::{Accounts, GroupRow, UserRow};

const ADMIN_GROUPS: &[&str] = &["sudo", "wheel", "admin", "root"];
/// Ids below 1000 belong to the system; 65534 and above is `nobody` and friends.
const REGULAR_IDS: std::ops::Range<u32> = 1000..65534;
const NO_LOGIN: &[&str] = &["nologin", "false", "sync", "shutdown", "halt"];

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(crate) struct Passwd {
    pub(crate) full_name: Option<String>,
    pub(crate) home: Option<String>,
    pub(crate) shell: Option<String>,
}

pub(crate) fn parse_passwd(text: &str) -> HashMap<String, Passwd> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|line| {
            let f: Vec<&str> = line.split(':').collect();
            if f.len() < 7 {
                return None;
            }
            let clean = |s: &str| {
                let s = s.split(',').next().unwrap_or("").trim();
                (!s.is_empty()).then(|| s.to_owned())
            };
            Some((
                f[0].to_owned(),
                Passwd {
                    full_name: clean(f[4]),
                    home: clean(f[5]),
                    shell: clean(f[6]),
                },
            ))
        })
        .collect()
}

pub(crate) fn can_login(shell: Option<&str>) -> Option<bool> {
    let shell = shell?;
    let program = shell.rsplit('/').next().unwrap_or(shell);
    Some(!NO_LOGIN.contains(&program))
}

pub(crate) fn user_kind(uid: &str, shell: Option<&str>) -> &'static str {
    let Ok(n) = uid.parse::<u32>() else {
        return "regular";
    };
    if n == 0 {
        "root"
    } else if !REGULAR_IDS.contains(&n) || can_login(shell) == Some(false) {
        "system"
    } else {
        "regular"
    }
}

pub(crate) fn group_kind(gid: &str, name: &str) -> &'static str {
    if ADMIN_GROUPS.contains(&name) {
        return "admin";
    }
    match gid.parse::<u32>() {
        Ok(n) if !REGULAR_IDS.contains(&n) => "system",
        _ => "regular",
    }
}

pub(crate) fn current_user() -> Option<String> {
    ["USER", "LOGNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .filter(|u| !u.is_empty())
}

fn passwd() -> HashMap<String, Passwd> {
    std::fs::read_to_string("/etc/passwd")
        .map(|t| parse_passwd(&t))
        .unwrap_or_default()
}

pub(crate) fn snapshot() -> Accounts {
    let users = Users::new_with_refreshed_list();
    let group_list = Groups::new_with_refreshed_list();
    let passwd = passwd();
    let me = current_user();

    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_memory()
            .with_user(UpdateKind::OnlyIfNotSet),
    );
    let mut usage: HashMap<String, (u32, u64)> = HashMap::new();
    for p in sys
        .processes()
        .values()
        .filter(|p| p.thread_kind().is_none())
    {
        if let Some(uid) = p.user_id() {
            let e = usage.entry((**uid).to_string()).or_default();
            e.0 += 1;
            e.1 += p.memory();
        }
    }

    let mut rows: Vec<UserRow> = users
        .list()
        .iter()
        .map(|u| {
            let uid = (**u.id()).to_string();
            let gid = (*u.group_id()).to_string();
            let primary = group_list
                .list()
                .iter()
                .find(|g| (**g.id()).to_string() == gid)
                .map(|g| g.name().to_owned())
                .unwrap_or_else(|| gid.clone());
            let mut groups: Vec<String> = u.groups().iter().map(|g| g.name().to_owned()).collect();
            if !groups.contains(&primary) && primary != gid {
                groups.push(primary.clone());
            }
            groups.sort();
            groups.dedup();
            let pw = passwd.get(u.name()).cloned().unwrap_or_default();
            let (processes, memory) = usage.get(&uid).copied().unwrap_or((0, 0));
            UserRow {
                name: u.name().to_owned(),
                kind: user_kind(&uid, pw.shell.as_deref()),
                admin: groups.iter().any(|g| ADMIN_GROUPS.contains(&g.as_str())),
                can_login: can_login(pw.shell.as_deref()),
                current: me.as_deref() == Some(u.name()),
                uid,
                primary_group: primary,
                groups,
                full_name: pw.full_name,
                home: pw.home,
                shell: pw.shell,
                processes,
                memory,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        let rank = |k: &str| match k {
            "regular" => 0,
            "root" => 1,
            _ => 2,
        };
        rank(a.kind)
            .cmp(&rank(b.kind))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let members = membership(&rows);
    let mut groups: Vec<GroupRow> = group_list
        .list()
        .iter()
        .map(|g| {
            let gid = (**g.id()).to_string();
            GroupRow {
                kind: group_kind(&gid, g.name()),
                admin: ADMIN_GROUPS.contains(&g.name()),
                members: members.get(g.name()).cloned().unwrap_or_default(),
                name: g.name().to_owned(),
                gid,
            }
        })
        .collect();
    groups.sort_by_key(|g| g.name.to_lowercase());

    Accounts {
        users: rows,
        groups,
    }
}

pub(crate) fn membership(users: &[UserRow]) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for u in users {
        for g in &u.groups {
            out.entry(g.clone()).or_default().push(u.name.clone());
        }
    }
    for list in out.values_mut() {
        list.sort();
        list.dedup();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
talha:x:1000:1000:Talha M,,,:/home/talha:/bin/bash
# a comment
broken-line
";

    fn user(name: &str, groups: &[&str]) -> UserRow {
        UserRow {
            name: name.into(),
            uid: "1000".into(),
            primary_group: name.into(),
            groups: groups.iter().map(|g| g.to_string()).collect(),
            full_name: None,
            home: None,
            shell: None,
            kind: "regular",
            admin: false,
            can_login: None,
            current: false,
            processes: 0,
            memory: 0,
        }
    }

    #[test]
    fn passwd_lines_give_name_home_and_shell() {
        let p = parse_passwd(PASSWD);
        assert_eq!(p.len(), 3);
        assert_eq!(p["talha"].full_name.as_deref(), Some("Talha M"));
        assert_eq!(p["talha"].home.as_deref(), Some("/home/talha"));
        assert_eq!(p["daemon"].shell.as_deref(), Some("/usr/sbin/nologin"));
        assert_eq!(p["root"].full_name.as_deref(), Some("root"));
    }

    #[test]
    fn login_follows_the_shell() {
        assert_eq!(can_login(Some("/bin/bash")), Some(true));
        assert_eq!(can_login(Some("/usr/sbin/nologin")), Some(false));
        assert_eq!(can_login(Some("/bin/false")), Some(false));
        assert_eq!(can_login(None), None);
    }

    #[test]
    fn users_are_classified_by_id_and_shell() {
        assert_eq!(user_kind("0", Some("/bin/bash")), "root");
        assert_eq!(user_kind("1", None), "system");
        assert_eq!(user_kind("1000", Some("/bin/bash")), "regular");
        assert_eq!(user_kind("1001", Some("/usr/sbin/nologin")), "system");
        assert_eq!(user_kind("65534", None), "system");
    }

    #[test]
    fn groups_are_classified_and_admin_groups_stand_out() {
        assert_eq!(group_kind("27", "sudo"), "admin");
        assert_eq!(group_kind("100", "users"), "system");
        assert_eq!(group_kind("1000", "talha"), "regular");
    }

    #[test]
    fn membership_is_the_inverse_of_each_users_groups() {
        let users = vec![
            user("a", &["sudo", "docker"]),
            user("b", &["docker", "docker"]),
        ];
        let m = membership(&users);
        assert_eq!(m["docker"], ["a", "b"]);
        assert_eq!(m["sudo"], ["a"]);
        assert!(!m.contains_key("nope"));
    }
}
