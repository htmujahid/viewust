use std::path::Path;

use super::model::{DockerDf, VirtOverview, VirtSlice};
use crate::common::cmd::{run, run_any};
use crate::common::sysfs::read;

/// The virtualization extension named in `/proc/cpuinfo`'s flags line.
pub(crate) fn cpu_extension(flags: &str) -> Option<&'static str> {
    let has = |f: &str| flags.split_whitespace().any(|w| w == f);
    if has("vmx") {
        Some("Intel VT-x")
    } else if has("svm") {
        Some("AMD-V")
    } else {
        None
    }
}

/// `docker system df --format "{{.Type}}\t{{.TotalCount}}\t{{.Size}}\t{{.Reclaimable}}"`.
pub(crate) fn parse_docker_df(text: &str) -> Vec<DockerDf> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            Some(DockerDf {
                kind: f.next()?.to_owned(),
                count: f.next()?.to_owned(),
                size: f.next()?.to_owned(),
                reclaimable: f.next()?.to_owned(),
            })
        })
        .collect()
}

fn number(path: impl AsRef<Path>) -> Option<u64> {
    read(path.as_ref())?.parse().ok()
}

/// Sums a set of cgroup directories into one line: how many, their processes and memory.
pub(crate) fn sum_slice(name: &str, dirs: &[std::path::PathBuf]) -> Option<VirtSlice> {
    if dirs.is_empty() {
        return None;
    }
    let sum = |file: &str| {
        let values: Vec<u64> = dirs.iter().filter_map(|d| number(d.join(file))).collect();
        (!values.is_empty()).then(|| values.iter().sum())
    };
    Some(VirtSlice {
        name: name.to_owned(),
        groups: dirs.len(),
        pids: sum("pids.current"),
        memory: sum("memory.current"),
    })
}

fn detect(arg: Option<&str>) -> Option<String> {
    let args: Vec<&str> = arg.into_iter().collect();
    run_any("systemd-detect-virt", &args)
        .map(|t| t.trim().to_owned())
        .filter(|v| !v.is_empty() && v != "none")
}

pub(crate) fn overview() -> VirtOverview {
    let flags = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("flags"))
                .map(str::to_owned)
        })
        .unwrap_or_default();
    let modules = std::fs::read_to_string("/proc/modules").unwrap_or_default();
    let kvm_module = ["kvm_intel", "kvm_amd"].into_iter().find(|m| {
        modules
            .lines()
            .any(|l| l.split_whitespace().next() == Some(m))
    });
    let nested = kvm_module.and_then(|m| {
        read(format!("/sys/module/{m}/parameters/nested")).map(|v| v == "Y" || v == "1")
    });

    // Running container roots are overlay filesystems; counting them needs no daemon.
    let overlay_mounts = std::fs::read_to_string("/proc/self/mountinfo")
        .map(|t| {
            t.lines()
                .filter(|l| {
                    l.split(" - ")
                        .nth(1)
                        .and_then(|rest| rest.split_whitespace().next())
                        == Some("overlay")
                })
                .count()
        })
        .unwrap_or(0);

    let docker_df = ["docker", "podman"]
        .into_iter()
        .find_map(|bin| {
            run(
                bin,
                &[
                    "system",
                    "df",
                    "--format",
                    "{{.Type}}\t{{.TotalCount}}\t{{.Size}}\t{{.Reclaimable}}",
                ],
            )
        })
        .map(|t| parse_docker_df(&t))
        .unwrap_or_default();

    let cg = Path::new("/sys/fs/cgroup");
    let mut slices = Vec::new();
    if cg.join("machine.slice").is_dir() {
        let guests: Vec<_> = std::fs::read_dir(cg.join("machine.slice"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.path())
            .collect();
        if guests.is_empty() {
            slices.extend(sum_slice(
                "machine.slice (no guests)",
                &[cg.join("machine.slice")],
            ));
        } else {
            slices.extend(sum_slice("machine.slice guests", &guests));
        }
    }
    let docker_scopes: Vec<_> = std::fs::read_dir(cg.join("system.slice"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.starts_with("docker-") && name.ends_with(".scope")
        })
        .map(|e| e.path())
        .collect();
    slices.extend(sum_slice("docker containers", &docker_scopes));

    VirtOverview {
        cpu: cpu_extension(&flags),
        kvm_device: Path::new("/dev/kvm").exists(),
        kvm_module: kvm_module.map(str::to_owned),
        nested,
        inside_vm: detect(None),
        inside_container: detect(Some("--container")),
        overlay_mounts,
        ip_forward: read("/proc/sys/net/ipv4/ip_forward").as_deref() == Some("1"),
        docker_df,
        slices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cpu_extension_is_read_from_the_flags_line() {
        assert_eq!(cpu_extension("flags : fpu vme vmx est"), Some("Intel VT-x"));
        assert_eq!(cpu_extension("flags : fpu svm"), Some("AMD-V"));
        assert_eq!(cpu_extension("flags : fpu smx"), None, "smx is not vmx");
    }

    #[test]
    fn docker_df_lines_split_into_kinds() {
        let d =
            parse_docker_df("Images\t4\t1.809GB\t1.731GB (95%)\nContainers\t3\t24.5MB\t0B (0%)\n");
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].kind, "Images");
        assert_eq!(d[0].reclaimable, "1.731GB (95%)");
    }

    #[test]
    fn slices_sum_their_groups() {
        let root = std::env::temp_dir().join(format!("viewust-virt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (name, pids, mem) in [("a.scope", "3", "1000"), ("b.scope", "2", "500")] {
            let d = root.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("pids.current"), pids).unwrap();
            std::fs::write(d.join("memory.current"), mem).unwrap();
        }
        let dirs: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        let s = sum_slice("test", &dirs).unwrap();
        assert_eq!((s.groups, s.pids, s.memory), (2, Some(5), Some(1500)));
        assert!(sum_slice("empty", &[]).is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn whatever_the_machine_the_answer_holds_together() {
        // x86 desktops have vmx/svm; arm machines have /dev/kvm with neither flag;
        // CI runners may have nothing. Only the shape is guaranteed.
        let v = overview();
        for s in &v.slices {
            assert!(s.groups > 0, "{} listed with no groups", s.name);
        }
        for d in &v.docker_df {
            assert!(!d.kind.is_empty());
        }
    }
}
