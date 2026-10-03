use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::model::*;
use super::sources::*;
use crate::common::sysfs::*;
use std::collections::HashMap;
use sysinfo::{Disks, System};

struct Tables {
    sys: System,
    at: Instant,
    disks: HashMap<String, (u64, u64, u64)>,
    net: HashMap<String, (u64, u64)>,
}

impl Tables {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        std::thread::sleep(Duration::from_millis(250));
        Self {
            sys,
            at: Instant::now(),
            disks: diskstats(),
            net: netstats(),
        }
    }
}

#[derive(Clone, Default)]
pub struct MonitorService(Arc<Mutex<Option<Tables>>>);

impl MonitorService {
    pub fn sample(&self) -> Sample {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let st = guard.get_or_insert_with(Tables::new);
        st.sys.refresh_cpu_all();
        st.sys.refresh_memory();

        let elapsed = st.at.elapsed().as_secs_f64().max(0.05);
        st.at = Instant::now();

        let cores: Vec<f32> = st.sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let freq = if cores.is_empty() {
            0.0
        } else {
            st.sys
                .cpus()
                .iter()
                .map(|c| c.frequency() as f32)
                .sum::<f32>()
                / cores.len() as f32
        };
        let load = System::load_average();
        let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let cached = meminfo
            .lines()
            .find_map(|l| l.strip_prefix("Cached:"))
            .and_then(|v| v.trim().trim_end_matches(" kB").parse::<u64>().ok())
            .unwrap_or(0)
            * 1024;

        let now_disks = diskstats();
        let mut disks: Vec<DiskRate> = now_disks
            .iter()
            .filter_map(|(name, (r, w, busy))| {
                let (pr, pw, pb) = st.disks.get(name)?;
                Some(DiskRate {
                    name: name.clone(),
                    model: read(format!("/sys/block/{name}/device/model")),
                    read_bps: r.saturating_sub(*pr) as f64 * 512.0 / elapsed,
                    write_bps: w.saturating_sub(*pw) as f64 * 512.0 / elapsed,
                    busy: (busy.saturating_sub(*pb) as f64 / (elapsed * 1000.0) * 100.0).min(100.0),
                })
            })
            .collect();
        disks.sort_by(|a, b| a.name.cmp(&b.name));
        st.disks = now_disks;

        let route = default_interface();
        let now_net = netstats();
        let mut net: Vec<NetRate> = now_net
            .iter()
            .filter(|(name, _)| {
                read(format!("/sys/class/net/{name}/operstate")).as_deref() == Some("up")
            })
            .filter_map(|(name, (rx, tx))| {
                let (prx, ptx) = st.net.get(name)?;
                let wifi = std::path::Path::new(&format!("/sys/class/net/{name}/wireless"))
                    .exists()
                    || std::path::Path::new(&format!("/sys/class/net/{name}/phy80211")).exists();
                Some(NetRate {
                    name: name.clone(),
                    kind: if wifi { "wifi" } else { "ethernet" },
                    rx_bps: rx.saturating_sub(*prx) as f64 / elapsed,
                    tx_bps: tx.saturating_sub(*ptx) as f64 / elapsed,
                    rx_total: *rx,
                    tx_total: *tx,
                    speed_mbps: read(format!("/sys/class/net/{name}/speed"))
                        .and_then(|v| v.parse::<i64>().ok())
                        .filter(|s| *s > 0)
                        .map(|s| s as u32),
                    default_route: route.as_deref() == Some(name.as_str()),
                })
            })
            .collect();
        net.sort_by(|a, b| {
            b.default_route
                .cmp(&a.default_route)
                .then(a.name.cmp(&b.name))
        });
        st.net = now_net;

        let volumes = Disks::new_with_refreshed_list()
            .iter()
            .filter(|d| d.total_space() > 0)
            .map(|d| Volume {
                mount: d.mount_point().to_string_lossy().into_owned(),
                file_system: d.file_system().to_string_lossy().into_owned(),
                total: d.total_space(),
                used: d.total_space() - d.available_space(),
            })
            .collect();

        Sample {
            t: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            cpu: Cpu {
                total: st.sys.global_cpu_usage(),
                cores,
                freq_mhz: freq,
                temperature: package_temperature(),
                load: [load.one, load.five, load.fifteen],
            },
            memory: Memory {
                total: st.sys.total_memory(),
                used: st.sys.used_memory(),
                available: st.sys.available_memory(),
                cached,
                swap_total: st.sys.total_swap(),
                swap_used: st.sys.used_swap(),
            },
            disks,
            volumes,
            gpus: gpus(),
            net,
        }
    }
}
