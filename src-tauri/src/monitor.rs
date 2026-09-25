//! One-second live samples of CPU, memory, drives, graphics cards and network.
//!
//! The monitor page polls `monitor_sample` once a second. Counters that only
//! ever grow (disk sectors, network bytes) are turned into rates here, using
//! the previous call's values kept in a process-wide state.

use crate::detail::read;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, System};

struct State {
    sys: System,
    at: Instant,
    disks: HashMap<String, (u64, u64, u64)>, // sectors read, sectors written, ms busy
    net: HashMap<String, (u64, u64)>,        // bytes received, bytes sent
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

#[derive(Serialize)]
pub struct Cpu {
    /// Machine-wide load, 0–100.
    total: f32,
    /// Load of each hardware thread, 0–100.
    cores: Vec<f32>,
    freq_mhz: f32,
    temperature: Option<f32>,
    load: [f64; 3],
}

#[derive(Serialize)]
pub struct Memory {
    total: u64,
    used: u64,
    available: u64,
    cached: u64,
    swap_total: u64,
    swap_used: u64,
}

#[derive(Serialize)]
pub struct DiskRate {
    name: String,
    model: Option<String>,
    read_bps: f64,
    write_bps: f64,
    /// Share of the last second the drive spent busy, 0–100.
    busy: f64,
}

#[derive(Serialize)]
pub struct Volume {
    mount: String,
    file_system: String,
    total: u64,
    used: u64,
}

#[derive(Serialize)]
pub struct GpuSample {
    name: String,
    util: Option<f64>,
    memory_used: Option<u64>,
    memory_total: Option<u64>,
    temperature: Option<f64>,
    power: Option<f64>,
    power_limit: Option<f64>,
    core_mhz: Option<f64>,
    memory_mhz: Option<f64>,
    fan: Option<f64>,
}

#[derive(Serialize)]
pub struct NetRate {
    name: String,
    kind: &'static str,
    rx_bps: f64,
    tx_bps: f64,
    rx_total: u64,
    tx_total: u64,
    speed_mbps: Option<u32>,
    default_route: bool,
}

#[derive(Serialize)]
pub struct Sample {
    /// Milliseconds since the Unix epoch.
    t: u64,
    cpu: Cpu,
    memory: Memory,
    disks: Vec<DiskRate>,
    volumes: Vec<Volume>,
    gpus: Vec<GpuSample>,
    net: Vec<NetRate>,
}

fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        // CPU load is a difference between two readings; take one now so the
        // first real sample already has a meaningful value.
        std::thread::sleep(Duration::from_millis(250));
        Mutex::new(State { sys, at: Instant::now(), disks: diskstats(), net: netstats() })
    })
}

fn diskstats() -> HashMap<String, (u64, u64, u64)> {
    let text = std::fs::read_to_string("/proc/diskstats").unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let name = f.get(2)?;
            // whole drives only: partitions have no /sys/block entry of their own
            if !std::path::Path::new(&format!("/sys/block/{name}")).exists() {
                return None;
            }
            if ["loop", "ram", "zram", "dm-", "sr", "md"].iter().any(|p| name.starts_with(p)) {
                return None;
            }
            Some((name.to_string(), (f.get(5)?.parse().ok()?, f.get(9)?.parse().ok()?, f.get(12)?.parse().ok()?)))
        })
        .collect()
}

fn netstats() -> HashMap<String, (u64, u64)> {
    let mut out = HashMap::new();
    for e in std::fs::read_dir("/sys/class/net").into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let p = e.path();
        // physical adapters only: skip loopback, bridges, containers, tunnels
        if name == "lo" || !p.join("device").exists() {
            continue;
        }
        let n = |f: &str| read(p.join("statistics").join(f)).and_then(|v| v.parse::<u64>().ok());
        if let (Some(rx), Some(tx)) = (n("rx_bytes"), n("tx_bytes")) {
            out.insert(name, (rx, tx));
        }
    }
    out
}

fn default_interface() -> Option<String> {
    crate::connection::default_route().map(|r| r.interface)
}

fn gpus() -> Vec<GpuSample> {
    let num = |s: &str| s.trim().parse::<f64>().ok();
    let nvidia: Vec<GpuSample> = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,power.limit,clocks.gr,clocks.mem,fan.speed",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| {
                    let f: Vec<&str> = l.split(", ").collect();
                    let g = |i: usize| f.get(i).and_then(|v| num(v));
                    GpuSample {
                        name: f.first().unwrap_or(&"GPU").trim().to_owned(),
                        util: g(1),
                        memory_used: g(2).map(|m| (m as u64) << 20),
                        memory_total: g(3).map(|m| (m as u64) << 20),
                        temperature: g(4),
                        power: g(5),
                        power_limit: g(6),
                        core_mhz: g(7),
                        memory_mhz: g(8),
                        fan: g(9),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    if !nvidia.is_empty() {
        return nvidia;
    }

    // AMD cards report the same figures through sysfs.
    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/drm").into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let dev = e.path().join("device");
        if !name.starts_with("card") || name.contains('-') || !dev.join("gpu_busy_percent").exists() {
            continue;
        }
        let n = |f: &str| read(dev.join(f)).and_then(|v| v.parse::<f64>().ok());
        let hw = std::fs::read_dir(dev.join("hwmon")).ok().and_then(|mut d| d.next()).and_then(|e| e.ok()).map(|e| e.path());
        let h = |f: &str| hw.as_ref().and_then(|p| read(p.join(f))).and_then(|v| v.parse::<f64>().ok());
        out.push(GpuSample {
            name: format!("AMD graphics ({name})"),
            util: n("gpu_busy_percent"),
            memory_used: n("mem_info_vram_used").map(|v| v as u64),
            memory_total: n("mem_info_vram_total").map(|v| v as u64),
            temperature: h("temp1_input").map(|t| t / 1000.0),
            power: h("power1_average").map(|p| p / 1e6),
            power_limit: h("power1_cap").map(|p| p / 1e6),
            core_mhz: None,
            memory_mhz: None,
            fan: None,
        });
    }
    out
}

fn package_temperature() -> Option<f32> {
    for e in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let p = e.path();
        let name = read(p.join("name"))?;
        if name != "coretemp" && name != "k10temp" {
            continue;
        }
        for i in 1..32 {
            let label = read(p.join(format!("temp{i}_label")));
            if matches!(label.as_deref(), Some("Package id 0") | Some("Tctl") | Some("Tdie")) {
                return read(p.join(format!("temp{i}_input"))).and_then(|v| v.parse::<f32>().ok()).map(|v| v / 1000.0);
            }
        }
    }
    None
}

#[tauri::command]
pub fn monitor_sample() -> Sample {
    let mut st = state().lock().unwrap_or_else(|e| e.into_inner());
    st.sys.refresh_cpu_all();
    st.sys.refresh_memory();

    let elapsed = st.at.elapsed().as_secs_f64().max(0.05);
    st.at = Instant::now();

    let cores: Vec<f32> = st.sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let freq = if cores.is_empty() { 0.0 } else { st.sys.cpus().iter().map(|c| c.frequency() as f32).sum::<f32>() / cores.len() as f32 };
    let load = System::load_average();
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let cached = meminfo
        .lines()
        .find_map(|l| l.strip_prefix("Cached:"))
        .and_then(|v| v.trim().trim_end_matches(" kB").parse::<u64>().ok())
        .unwrap_or(0)
        * 1024;

    // drives
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

    // network
    let route = default_interface();
    let now_net = netstats();
    let mut net: Vec<NetRate> = now_net
        .iter()
        .filter(|(name, _)| read(format!("/sys/class/net/{name}/operstate")).as_deref() == Some("up"))
        .filter_map(|(name, (rx, tx))| {
            let (prx, ptx) = st.net.get(name)?;
            let wifi = std::path::Path::new(&format!("/sys/class/net/{name}/wireless")).exists()
                || std::path::Path::new(&format!("/sys/class/net/{name}/phy80211")).exists();
            Some(NetRate {
                name: name.clone(),
                kind: if wifi { "wifi" } else { "ethernet" },
                rx_bps: rx.saturating_sub(*prx) as f64 / elapsed,
                tx_bps: tx.saturating_sub(*ptx) as f64 / elapsed,
                rx_total: *rx,
                tx_total: *tx,
                speed_mbps: read(format!("/sys/class/net/{name}/speed")).and_then(|v| v.parse::<i64>().ok()).filter(|s| *s > 0).map(|s| s as u32),
                default_route: route.as_deref() == Some(name.as_str()),
            })
        })
        .collect();
    net.sort_by(|a, b| b.default_route.cmp(&a.default_route).then(a.name.cmp(&b.name)));
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
        t: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
        cpu: Cpu { total: st.sys.global_cpu_usage(), cores, freq_mhz: freq, temperature: package_temperature(), load: [load.one, load.five, load.fifteen] },
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
