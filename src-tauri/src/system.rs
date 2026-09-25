//! The computer's internals: motherboard, processor, memory, drives, graphics
//! cards, network and sound adapters, and what is known about power.
//!
//! Everything is read without administrator rights. The two things the
//! operating system won't hand over freely are called out in the results:
//! individual memory modules (see `read_memory_modules`) and the power supply,
//! which never reports itself.

use crate::detail::{format_bytes, read, Detail, Details};
use crate::report::{lookup_text, PCI_IDS};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Serialize)]
pub struct Component {
    id: String,
    kind: &'static str,
    name: String,
    subtitle: Option<String>,
    details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    computer_name: String,
    components: Vec<Component>,
}

#[derive(Serialize)]
pub struct MemoryModules {
    modules: Vec<Component>,
    /// Total memory slots on the board, including empty ones.
    slots: u32,
}

#[tauri::command]
pub fn system_info() -> SystemInfo {
    let pci = Pci::load();
    let mut components = vec![board(&pci), cpu(), memory()];
    components.extend(gpus(&pci));
    components.extend(storage());
    components.extend(network(&pci));
    components.extend(sound(&pci));
    components.push(power());
    SystemInfo {
        computer_name: sysinfo::System::host_name().unwrap_or_else(|| "This computer".into()),
        components,
    }
}

// ------------------------------------------------------------------ helpers

fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn decimal_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1e12 {
        format!("{:.1} TB", b / 1e12)
    } else if b >= 1e9 {
        format!("{:.0} GB", b / 1e9)
    } else {
        format!("{:.0} MB", b / 1e6)
    }
}

fn watts(microwatts: u64) -> String {
    format!("{:.0} W", microwatts as f64 / 1e6)
}

/// Temperature (°C) from the first hwmon chip with one of these names.
fn hwmon_temp(chips: &[&str], label: Option<&str>) -> Option<f64> {
    for e in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let p = e.path();
        if !chips.contains(&read(p.join("name"))?.as_str()) {
            continue;
        }
        for i in 1..32 {
            let want = label.map_or(true, |l| read(p.join(format!("temp{i}_label"))).as_deref() == Some(l));
            if want {
                if let Some(v) = read(p.join(format!("temp{i}_input"))).and_then(|v| v.parse::<f64>().ok()) {
                    return Some(v / 1000.0);
                }
            }
        }
    }
    None
}

fn dmi(file: &str) -> Option<String> {
    read(format!("/sys/class/dmi/id/{file}")).filter(|v| {
        let l = v.to_lowercase();
        !matches!(l.as_str(), "default string" | "to be filled by o.e.m." | "not specified" | "none")
    })
}

// ---------------------------------------------------------------------- PCI

struct PciDevice {
    slot: String,
    class: u32,
    vendor: String,
    device: String,
    vendor_name: Option<String>,
    device_name: Option<String>,
}

struct Pci(Vec<PciDevice>);

impl Pci {
    fn load() -> Self {
        let text = PCI_IDS.iter().find_map(|f| std::fs::read_to_string(f).ok()).unwrap_or_default();
        let hex = |p: &Path, f: &str| read(p.join(f)).map(|v| v.trim_start_matches("0x").to_owned());
        let mut out: Vec<PciDevice> = std::fs::read_dir("/sys/bus/pci/devices")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let (vendor, device) = (hex(&p, "vendor")?, hex(&p, "device")?);
                let (vendor_name, device_name) = lookup_text(&text, &vendor, Some(&device));
                Some(PciDevice {
                    slot: e.file_name().to_string_lossy().into_owned(),
                    class: u32::from_str_radix(&hex(&p, "class")?, 16).ok()?,
                    vendor,
                    device,
                    vendor_name,
                    device_name,
                })
            })
            .collect();
        out.sort_by(|a, b| a.slot.cmp(&b.slot));
        Self(out)
    }

    fn at(&self, slot: &str) -> Option<&PciDevice> {
        self.0.iter().find(|d| d.slot == slot)
    }

    fn by_class(&self, class: u32) -> Option<&PciDevice> {
        self.0.iter().find(|d| d.class >> 8 == class)
    }
}

fn pci_class_name(class: u32) -> &'static str {
    match class >> 8 {
        0x0100 => "SCSI controller",
        0x0101 => "IDE controller",
        0x0106 => "SATA controller",
        0x0108 => "NVMe controller",
        0x0200 => "Ethernet controller",
        0x0280 => "Network controller",
        0x0300 => "VGA controller",
        0x0302 => "3D controller",
        0x0401 => "Audio device",
        0x0403 => "Audio controller",
        0x0500 => "RAM controller",
        0x0600 => "Host bridge",
        0x0601 => "ISA/LPC bridge",
        0x0604 => "PCI bridge",
        0x0c03 => "USB controller",
        0x0c05 => "SMBus controller",
        0x0780 => "Communication controller",
        0x0880 => "System peripheral",
        0x1180 => "Signal processing controller",
        _ => "Other device",
    }
}

fn pci_model(d: &PciDevice) -> String {
    d.device_name.clone().unwrap_or_else(|| format!("{}:{}", d.vendor, d.device))
}

// -------------------------------------------------------------- motherboard

fn chassis_name(code: &str) -> &'static str {
    match code.parse::<u32>().unwrap_or(0) {
        3 => "Desktop",
        4 => "Low-profile desktop",
        5 => "Pizza box",
        6 => "Mini tower",
        7 => "Tower",
        8 => "Portable",
        9 => "Laptop",
        10 => "Notebook",
        13 => "All-in-one",
        15 => "Space-saving",
        16 => "Lunch box",
        17 => "Server",
        23 => "Rack server",
        30 => "Tablet",
        31 => "Convertible",
        35 => "Mini PC",
        36 => "Stick PC",
        _ => "Other",
    }
}

fn board(pci: &Pci) -> Component {
    let mut d = Details::new();
    d.add_opt("Motherboard", "Manufacturer", dmi("board_vendor"));
    d.add_opt("Motherboard", "Model", dmi("board_name"));
    d.add_opt("Motherboard", "Revision", dmi("board_version"));
    d.add_opt("System", "Manufacturer", dmi("sys_vendor"));
    d.add_opt("System", "Model", dmi("product_name"));
    d.add_opt("System", "Family", dmi("product_family"));
    d.add_opt("System", "Case type", read("/sys/class/dmi/id/chassis_type").map(|c| chassis_name(&c)));
    d.add_opt("BIOS / UEFI", "Vendor", dmi("bios_vendor"));
    d.add_opt("BIOS / UEFI", "Version", dmi("bios_version"));
    d.add_opt("BIOS / UEFI", "Date", dmi("bios_date"));
    d.add_opt("BIOS / UEFI", "Release", dmi("bios_release"));
    d.add(
        "BIOS / UEFI",
        "Boot mode",
        if Path::new("/sys/firmware/efi").exists() { "UEFI" } else { "Legacy BIOS" },
    );
    if let Some(c) = pci.by_class(0x0601) {
        d.add("Chipset", "Chipset", pci_model(c));
        d.add_opt("Chipset", "Vendor", c.vendor_name.clone());
    }
    if let Some(h) = pci.by_class(0x0600) {
        d.add("Chipset", "Host bridge", pci_model(h));
    }
    d.add("Chipset", "PCI devices", format!("{} on the board (see technical details)", pci.0.len()));
    Component {
        id: "sys:board".into(),
        kind: "board",
        name: dmi("board_name").unwrap_or_else(|| "Motherboard".into()),
        subtitle: dmi("board_vendor"),
        details: d.finish(),
    }
}

// ---------------------------------------------------------------------- CPU

fn cpuinfo() -> std::collections::HashMap<String, String> {
    let text = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    text.split("\n\n")
        .next()
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect()
}

fn cpu() -> Component {
    let info = cpuinfo();
    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_all();
    let brand = info
        .get("model name")
        .cloned()
        .or_else(|| sys.cpus().first().map(|c| c.brand().trim().to_owned()))
        .unwrap_or_else(|| "Processor".into());

    let mut d = Details::new();
    d.add("Processor", "Model", &brand);
    d.add_opt("Processor", "Vendor", info.get("vendor_id").cloned());
    d.add_opt(
        "Processor",
        "Family / model / stepping",
        match (info.get("cpu family"), info.get("model"), info.get("stepping")) {
            (Some(a), Some(b), Some(c)) => Some(format!("{a} / {b} / {c}")),
            _ => None,
        },
    );
    d.add_opt("Processor", "Microcode", info.get("microcode").cloned());
    d.add("Processor", "Architecture", sysinfo::System::cpu_arch());
    d.add_opt(
        "Cores",
        "Physical cores",
        sysinfo::System::physical_core_count().map(|n| n.to_string()),
    );
    d.add("Cores", "Threads", sys.cpus().len().to_string());

    let cpu0 = Path::new("/sys/devices/system/cpu/cpu0");
    let khz = |f: &str| read(cpu0.join("cpufreq").join(f)).and_then(|v| v.parse::<f64>().ok());
    if let (Some(lo), Some(hi)) = (khz("cpuinfo_min_freq"), khz("cpuinfo_max_freq")) {
        d.add("Clock", "Range", format!("{:.1} – {:.1} GHz", lo / 1e6, hi / 1e6));
    }
    if !sys.cpus().is_empty() {
        let avg = sys.cpus().iter().map(|c| c.frequency()).sum::<u64>() as f64 / sys.cpus().len() as f64;
        d.add("Clock", "Now (average)", format!("{:.2} GHz", avg / 1000.0));
    }
    d.add_opt("Clock", "Governor", read(cpu0.join("cpufreq/scaling_governor")));
    for i in 0..6 {
        let c = cpu0.join(format!("cache/index{i}"));
        if let (Some(level), Some(kind), Some(size)) = (read(c.join("level")), read(c.join("type")), read(c.join("size"))) {
            d.add("Cache", format!("L{level} {}", kind.to_lowercase()), size);
        }
    }

    let rapl = |n: u8| read(format!("/sys/class/powercap/intel-rapl:0/constraint_{n}_power_limit_uw")).and_then(|v| v.parse::<u64>().ok());
    d.add_opt("Power and heat", "Sustained power limit (PL1)", rapl(0).map(watts));
    d.add_opt("Power and heat", "Boost power limit (PL2)", rapl(1).map(watts));
    d.add_opt(
        "Power and heat",
        "Temperature",
        hwmon_temp(&["coretemp"], Some("Package id 0"))
            .or_else(|| hwmon_temp(&["k10temp"], None))
            .map(|t| format!("{t:.0} °C")),
    );

    let flags = info.get("flags").cloned().unwrap_or_default();
    let has = |f: &str| flags.split_whitespace().any(|x| x == f);
    let sets: Vec<&str> = ["sse4_2", "avx", "avx2", "avx512f", "fma", "aes", "sha_ni"].into_iter().filter(|f| has(f)).collect();
    d.add("Features", "Instruction sets", sets.join(", ").to_uppercase());
    d.add(
        "Features",
        "Virtualization",
        if has("vmx") || has("svm") { "Supported" } else { "Not available" },
    );

    Component {
        id: "sys:cpu".into(),
        kind: "cpu",
        name: brand,
        subtitle: Some(format!(
            "{} cores · {} threads",
            sysinfo::System::physical_core_count().map_or("?".into(), |n| n.to_string()),
            sys.cpus().len()
        )),
        details: d.finish(),
    }
}

// ------------------------------------------------------------------- memory

fn meminfo() -> std::collections::HashMap<String, u64> {
    std::fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.to_owned(), v.trim().trim_end_matches(" kB").parse::<u64>().ok()? * 1024))
        })
        .collect()
}

fn memory() -> Component {
    let m = meminfo();
    let g = |k: &str| m.get(k).copied().unwrap_or(0);
    let total = g("MemTotal");
    let mut d = Details::new();
    d.add("Memory", "Usable", format_bytes(total));
    d.add("Memory", "In use", format_bytes(total.saturating_sub(g("MemAvailable"))));
    d.add("Memory", "Available", format_bytes(g("MemAvailable")));
    d.add("Memory", "Cached", format_bytes(g("Cached")));
    if g("SwapTotal") > 0 {
        d.add("Swap", "Size", format_bytes(g("SwapTotal")));
        d.add("Swap", "In use", format_bytes(g("SwapTotal") - g("SwapFree")));
    }
    d.add(
        "Memory modules",
        "Status",
        "Not read yet. Slot, speed, type and maker of each module are restricted to \
         administrators. Use “Read memory modules” to unlock them.",
    );
    Component {
        id: "sys:ram".into(),
        kind: "ram",
        name: "System memory".into(),
        subtitle: Some(format_bytes(total)),
        details: d.finish(),
    }
}

/// Reads each memory module with `dmidecode`, asking for administrator
/// permission through the desktop's own password prompt. Only runs when the
/// user asks for it.
#[tauri::command]
pub fn read_memory_modules() -> Result<MemoryModules, String> {
    let program = ["/usr/sbin/dmidecode", "/usr/bin/dmidecode", "/sbin/dmidecode"]
        .into_iter()
        .find(|p| Path::new(p).exists())
        .ok_or("dmidecode is not installed")?;
    let out = Command::new("pkexec")
        .args([program, "-t", "17"])
        .output()
        .map_err(|e| format!("could not ask for permission: {e}"))?;
    if !out.status.success() {
        return Err(match out.status.code() {
            Some(126) | Some(127) => "Permission was declined".into(),
            _ => "Permission was declined or the read failed".into(),
        });
    }
    Ok(parse_dmidecode(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_dmidecode(text: &str) -> MemoryModules {
    let mut modules = Vec::new();
    let mut slots = 0;
    for block in text.split("\n\n").filter(|b| b.contains("Memory Device")) {
        slots += 1;
        let field = |name: &str| {
            block
                .lines()
                .find_map(|l| l.trim().strip_prefix(&format!("{name}:")))
                .map(|v| v.trim().to_owned())
                .filter(|v| !matches!(v.as_str(), "" | "Unknown" | "Not Specified" | "None" | "[Empty]"))
        };
        let Some(size) = field("Size").filter(|s| !s.contains("No Module")) else {
            continue;
        };
        let locator = field("Locator").unwrap_or_else(|| format!("Slot {slots}"));
        let kind = field("Type");
        let speed = field("Configured Memory Speed").or_else(|| field("Speed"));
        let maker = field("Manufacturer");
        let part = field("Part Number");

        let mut d = Details::new();
        d.add("Module", "Slot", &locator);
        d.add_opt("Module", "Bank", field("Bank Locator"));
        d.add("Module", "Capacity", &size);
        d.add_opt("Module", "Type", kind.clone());
        d.add_opt("Module", "Form factor", field("Form Factor"));
        d.add_opt("Module", "Rank", field("Rank"));
        d.add_opt("Speed", "Running at", field("Configured Memory Speed"));
        d.add_opt("Speed", "Rated", field("Speed"));
        d.add_opt("Speed", "Voltage", field("Configured Voltage"));
        d.add_opt("Speed", "Data width", field("Data Width"));
        d.add_opt("Maker", "Manufacturer", maker.clone());
        d.add_opt("Maker", "Part number", part.clone());
        d.add_opt("Maker", "Serial number", field("Serial Number"));

        modules.push(Component {
            id: format!("sys:ram:{}", modules.len()),
            kind: "ram",
            name: module_name(maker, part, &size, kind.as_deref()),
            subtitle: Some(format!(
                "{locator} · {size}{}",
                speed.map(|s| format!(" · {s}")).unwrap_or_default()
            )),
            details: d.finish(),
        });
    }
    MemoryModules { modules, slots }
}

/// "Kingston KF548C38-16", or "16 GB DDR5" when the module reports no maker.
fn module_name(maker: Option<String>, part: Option<String>, size: &str, kind: Option<&str>) -> String {
    let name = [maker, part].into_iter().flatten().collect::<Vec<_>>().join(" ");
    if name.trim().is_empty() {
        format!("{size} {}", kind.unwrap_or_default()).trim().to_owned()
    } else {
        name.trim().to_owned()
    }
}

// ------------------------------------------------------------------ storage

fn storage() -> Vec<Component> {
    let Some(json) = run(
        "lsblk",
        &["-J", "-b", "-o", "NAME,SIZE,TYPE,TRAN,ROTA,MODEL,SERIAL,REV,VENDOR,WWN,MOUNTPOINTS,FSTYPE,LABEL,HOTPLUG,RM"],
    )
    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) else {
        return Vec::new();
    };

    let text = |v: &serde_json::Value, k: &str| v[k].as_str().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    let mut out = Vec::new();

    for disk in json["blockdevices"].as_array().into_iter().flatten() {
        let name = text(disk, "name").unwrap_or_default();
        let tran = text(disk, "tran").unwrap_or_default();
        // Internal fixed disks only: skip loop/ram/optical devices and USB sticks.
        if disk["type"] != "disk" || disk["hotplug"] == true || disk["rm"] == true || tran == "usb" {
            continue;
        }
        let size = disk["size"].as_u64().unwrap_or(0);
        let is_nvme = name.starts_with("nvme") || tran == "nvme";
        let spinning = disk["rota"] == true;
        let kind = if is_nvme { "nvme" } else if spinning { "hdd" } else { "ssd" };
        let interface = match tran.as_str() {
            "sata" | "ata" => "SATA",
            "nvme" => "NVMe",
            "sas" => "SAS",
            "" if is_nvme => "NVMe",
            "" => "Internal",
            other => other,
        };

        let udev = run("udevadm", &["info", "--query=property", &format!("--name=/dev/{name}")]).unwrap_or_default();
        let prop = |k: &str| udev.lines().find_map(|l| l.strip_prefix(&format!("{k}=")).map(str::to_owned));
        let model = prop("ID_MODEL").map(|m| m.replace('_', " ")).or_else(|| text(disk, "model")).unwrap_or_else(|| name.clone());
        let q = |f: &str| read(format!("/sys/block/{name}/queue/{f}"));

        let mut d = Details::new();
        d.add("Drive", "Model", &model);
        d.add_opt("Drive", "Vendor", text(disk, "vendor"));
        d.add_opt("Drive", "Serial number", prop("ID_SERIAL_SHORT").or_else(|| text(disk, "serial")));
        d.add_opt("Drive", "Firmware", prop("ID_REVISION").or_else(|| text(disk, "rev")));
        d.add_opt("Drive", "WWN", prop("ID_WWN").or_else(|| text(disk, "wwn")));
        d.add("Capacity", "Size", format!("{} ({})", decimal_size(size), format_bytes(size)));
        d.add(
            "Type",
            "Technology",
            match (kind, prop("ID_ATA_ROTATION_RATE_RPM")) {
                ("hdd", Some(rpm)) if rpm != "0" => format!("Hard disk · {rpm} rpm"),
                ("hdd", _) => "Hard disk".to_owned(),
                ("nvme", _) => "Solid-state drive (NVMe)".to_owned(),
                _ => "Solid-state drive".to_owned(),
            },
        );
        d.add("Type", "Interface", interface);
        d.add("Type", "Device", format!("/dev/{name}"));
        d.add_opt("Type", "Sector size", match (q("logical_block_size"), q("physical_block_size")) {
            (Some(l), Some(p)) => Some(format!("{l} B logical · {p} B physical")),
            _ => None,
        });
        let mut features = Vec::new();
        for (key, label) in [
            ("ID_ATA_FEATURE_SET_SMART_ENABLED", "SMART health monitoring"),
            ("ID_ATA_WRITE_CACHE_ENABLED", "Write cache"),
            ("ID_ATA_FEATURE_SET_APM_ENABLED", "Power management (APM)"),
            ("ID_ATA_FEATURE_SET_PM_ENABLED", "Power saving"),
        ] {
            if prop(key).as_deref() == Some("1") {
                features.push(label);
            }
        }
        if !features.is_empty() {
            d.add("Type", "Enabled features", features.join(", "));
        }
        if is_nvme {
            d.add_opt("Health", "Temperature", hwmon_temp(&["nvme"], None).map(|t| format!("{t:.0} °C")));
        }

        for part in disk["children"].as_array().into_iter().flatten() {
            let mounts: Vec<String> = part["mountpoints"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|m| m.as_str().map(str::to_owned))
                .collect();
            d.add(
                "Partitions",
                text(part, "name").unwrap_or_default(),
                [
                    Some(decimal_size(part["size"].as_u64().unwrap_or(0))),
                    text(part, "fstype"),
                    text(part, "label"),
                    (!mounts.is_empty()).then(|| format!("mounted at {}", mounts.join(", "))),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · "),
            );
        }

        out.push(Component {
            id: format!("sys:disk:{name}"),
            kind,
            name: model,
            subtitle: Some(format!("{} · {interface}", decimal_size(size))),
            details: d.finish(),
        });
    }
    out
}

// ---------------------------------------------------------------------- GPU

struct Smi {
    values: Vec<String>,
}

fn nvidia_smi() -> Vec<Smi> {
    run(
        "nvidia-smi",
        &[
            "--query-gpu=pci.bus_id,name,memory.total,memory.used,temperature.gpu,power.draw,power.limit,\
             clocks.gr,clocks.max.gr,clocks.mem,driver_version,vbios_version,fan.speed,utilization.gpu,\
             pcie.link.gen.current,pcie.link.gen.max,pcie.link.width.current,pcie.link.width.max",
            "--format=csv,noheader,nounits",
        ],
    )
    .map(|t| {
        t.lines()
            .map(|l| Smi { values: l.split(", ").map(|v| v.trim().to_owned()).collect() })
            .collect()
    })
    .unwrap_or_default()
}

fn gpus(pci: &Pci) -> Vec<Component> {
    let smi = nvidia_smi();
    let mut seen: Vec<std::path::PathBuf> = Vec::new();
    let mut out = Vec::new();

    for e in std::fs::read_dir("/sys/class/drm").into_iter().flatten().flatten() {
        let card = e.file_name().to_string_lossy().into_owned();
        if !card.starts_with("card") || card.contains('-') {
            continue;
        }
        let dev = e.path().join("device");
        let Ok(real) = std::fs::canonicalize(&dev) else { continue };
        if seen.contains(&real) {
            continue;
        }
        seen.push(real.clone());
        let slot = real.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let Some(p) = pci.at(&slot) else { continue };

        let mine = smi.iter().find(|s| s.values.first().map(|b| b.to_lowercase().ends_with(&slot)) == Some(true));
        let val = |i: usize| mine.and_then(|s| s.values.get(i)).filter(|v| !v.contains("N/A")).cloned();
        let model = val(1).unwrap_or_else(|| pci_model(p));
        let driver = std::fs::read_link(dev.join("driver"))
            .ok()
            .and_then(|l| l.file_name().map(|n| n.to_string_lossy().into_owned()));

        let mut d = Details::new();
        d.add("Graphics card", "Model", &model);
        d.add_opt("Graphics card", "Chip", p.device_name.clone());
        d.add_opt("Graphics card", "Vendor", p.vendor_name.clone());
        d.add_opt("Graphics card", "Driver", driver.clone().map(|dr| match val(10) {
            Some(v) => format!("{dr} {v}"),
            None => dr,
        }));
        d.add_opt("Graphics card", "Video BIOS", val(11));
        d.add("Graphics card", "PCI address", &slot);

        let vram = val(2).and_then(|v| v.parse::<u64>().ok()).map(|m| m << 20)
            .or_else(|| read(dev.join("mem_info_vram_total")).and_then(|v| v.parse().ok()));
        d.add_opt("Memory", "Video memory", vram.map(format_bytes));
        d.add_opt("Memory", "In use", val(3).and_then(|v| v.parse::<u64>().ok()).map(|m| format_bytes(m << 20)));

        let gen = |cur: Option<String>, w: Option<String>| match (cur, w) {
            (Some(g), Some(w)) => Some(format!("PCIe {g}.0 ×{w}")),
            _ => None,
        };
        d.add_opt("PCIe link", "Now", gen(val(14), val(16)).or_else(|| {
            Some(format!("{} ×{}", read(dev.join("current_link_speed"))?, read(dev.join("current_link_width"))?))
        }));
        d.add_opt("PCIe link", "Maximum", gen(val(15), val(17)).or_else(|| {
            Some(format!("{} ×{}", read(dev.join("max_link_speed"))?, read(dev.join("max_link_width"))?))
        }));

        d.add_opt("Live readings", "Temperature", val(4).map(|t| format!("{t} °C")));
        d.add_opt("Live readings", "Power draw", match (val(5), val(6)) {
            (Some(a), Some(b)) => Some(format!("{a} W of {b} W limit")),
            _ => None,
        });
        d.add_opt("Live readings", "Core clock", match (val(7), val(8)) {
            (Some(a), Some(b)) => Some(format!("{a} MHz (max {b} MHz)")),
            _ => None,
        });
        d.add_opt("Live readings", "Memory clock", val(9).map(|c| format!("{c} MHz")));
        d.add_opt("Live readings", "Fan", val(12).map(|f| format!("{f} %")));
        d.add_opt("Live readings", "Load", val(13).map(|f| format!("{f} %")));

        let connected: Vec<String> = std::fs::read_dir("/sys/class/drm")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|c| {
                let n = c.file_name().to_string_lossy().into_owned();
                let rest = n.strip_prefix(&format!("{card}-"))?.to_owned();
                (read(c.path().join("status")).as_deref() == Some("connected")).then_some(rest)
            })
            .collect();
        d.add("Outputs", "Connected displays", if connected.is_empty() { "None".into() } else { connected.join(", ") });

        out.push(Component {
            id: format!("sys:gpu:{card}"),
            kind: "gpu",
            name: model,
            subtitle: Some([vram.map(format_bytes), p.vendor_name.clone()].into_iter().flatten().collect::<Vec<_>>().join(" · ")),
            details: d.finish(),
        });
    }
    out
}

// ----------------------------------------------------------------- network

fn network(pci: &Pci) -> Vec<Component> {
    let nets = sysinfo::Networks::new_with_refreshed_list();
    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/net").into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let p = e.path();
        // physical adapters have a backing device; bridges, veth and tunnels don't
        if !p.join("device").exists() || name == "lo" {
            continue;
        }
        let wifi = p.join("wireless").exists() || p.join("phy80211").exists();
        let slot = std::fs::canonicalize(p.join("device")).ok().and_then(|r| r.file_name().map(|n| n.to_string_lossy().into_owned()));
        let model = slot.as_deref().and_then(|s| pci.at(s)).map(pci_model);
        let driver = std::fs::read_link(p.join("device/driver")).ok().and_then(|l| l.file_name().map(|n| n.to_string_lossy().into_owned()));

        let mut d = Details::new();
        d.add("Adapter", "Interface", &name);
        d.add("Adapter", "Type", if wifi { "Wi-Fi" } else { "Ethernet" });
        d.add_opt("Adapter", "Model", model.clone());
        d.add_opt("Adapter", "Vendor", slot.as_deref().and_then(|s| pci.at(s)).and_then(|x| x.vendor_name.clone()));
        d.add_opt("Adapter", "Driver", driver);
        d.add_opt("Adapter", "MAC address", read(p.join("address")));
        d.add_opt("Link", "State", read(p.join("operstate")));
        d.add_opt("Link", "Speed", read(p.join("speed")).and_then(|s| s.parse::<i64>().ok()).filter(|s| *s > 0).map(|s| {
            if s >= 1000 { format!("{} Gbit/s", s as f64 / 1000.0) } else { format!("{s} Mbit/s") }
        }));
        d.add_opt("Link", "Duplex", read(p.join("duplex")));
        d.add_opt("Link", "MTU", read(p.join("mtu")));
        if let Some(data) = nets.iter().find(|(n, _)| **n == name).map(|(_, d)| d) {
            let ips: Vec<String> = data.ip_networks().iter().map(|ip| ip.addr.to_string()).collect();
            d.add("Addresses", "IP", ips.join(", "));
            d.add("Traffic", "Received", format_bytes(data.total_received()));
            d.add("Traffic", "Sent", format_bytes(data.total_transmitted()));
        }
        out.push(Component {
            id: format!("sys:nic:{name}"),
            kind: "nic",
            name: model.unwrap_or_else(|| name.clone()),
            subtitle: Some(format!("{name} · {}", if wifi { "Wi-Fi" } else { "Ethernet" })),
            details: d.finish(),
        });
    }
    out
}

// ------------------------------------------------------------------- sound

fn sound(pci: &Pci) -> Vec<Component> {
    // HDMI audio on a graphics card shares its PCI slot (minus the function).
    let gpu_slots: Vec<String> = pci
        .0
        .iter()
        .filter(|d| d.class >> 16 == 0x03)
        .map(|d| d.slot.rsplit_once('.').map(|(a, _)| a.to_owned()).unwrap_or_default())
        .collect();

    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/sound").into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(n) = name.strip_prefix("card").and_then(|n| n.parse::<u32>().ok()) else { continue };
        if Path::new(&format!("/proc/asound/card{n}/usbid")).exists() {
            continue; // USB sound devices are listed as peripherals
        }
        let dev = e.path().join("device");
        let Some(slot) = std::fs::canonicalize(&dev).ok().and_then(|r| r.file_name().map(|f| f.to_string_lossy().into_owned())) else { continue };
        if gpu_slots.contains(&slot.rsplit_once('.').map(|(a, _)| a.to_owned()).unwrap_or_default()) {
            continue;
        }
        let codec = std::fs::read_to_string(format!("/proc/asound/card{n}/codec#0")).ok().and_then(|t| {
            t.lines().find_map(|l| l.strip_prefix("Codec:").map(|c| c.trim().to_owned()))
        });
        let ctrl = pci.at(&slot);

        let mut d = Details::new();
        d.add_opt("Audio chip", "Codec", codec.clone());
        d.add_opt("Audio chip", "Card", read(format!("/proc/asound/card{n}/id")));
        d.add_opt("Controller", "Model", ctrl.map(pci_model));
        d.add_opt("Controller", "Vendor", ctrl.and_then(|c| c.vendor_name.clone()));
        d.add("Controller", "PCI address", &slot);
        d.add_opt("Controller", "Driver", std::fs::read_link(dev.join("driver")).ok().and_then(|l| l.file_name().map(|f| f.to_string_lossy().into_owned())));
        if let Ok(pcm) = std::fs::read_to_string("/proc/asound/pcm") {
            for line in pcm.lines().filter(|l| l.starts_with(&format!("{n:02}-"))) {
                let parts: Vec<&str> = line.split(" : ").collect();
                if parts.len() >= 3 {
                    d.add("Streams", parts[0].trim(), format!("{} · {}", parts[1].trim(), parts[2..].join(", ")));
                }
            }
        }
        out.push(Component {
            id: format!("sys:audio:{n}"),
            kind: "soundcard",
            name: codec.unwrap_or_else(|| "Sound card".into()),
            subtitle: ctrl.map(pci_model),
            details: d.finish(),
        });
    }
    out
}

// -------------------------------------------------------------------- power

fn power() -> Component {
    let rapl = |n: u8| read(format!("/sys/class/powercap/intel-rapl:0/constraint_{n}_power_limit_uw")).and_then(|v| v.parse::<u64>().ok()).map(|u| u as f64 / 1e6);
    let gpu_limits: f64 = nvidia_smi().iter().filter_map(|s| s.values.get(6)?.parse::<f64>().ok()).sum();
    let gpu_draw: f64 = nvidia_smi().iter().filter_map(|s| s.values.get(5)?.parse::<f64>().ok()).sum();

    let mut d = Details::new();
    d.add(
        "Power supply",
        "Model and wattage",
        "Not reported. A power supply doesn't talk to the computer, so no software can read its \
         model, rating or health. Check the label on the unit itself.",
    );
    d.add_opt("What the computer does report", "CPU sustained limit", rapl(0).map(|w| format!("{w:.0} W")));
    d.add_opt("What the computer does report", "CPU boost limit", rapl(1).map(|w| format!("{w:.0} W")));
    if gpu_limits > 0.0 {
        d.add("What the computer does report", "Graphics card draw now", format!("{gpu_draw:.0} W"));
        d.add("What the computer does report", "Graphics card limit", format!("{gpu_limits:.0} W"));
    }
    if gpu_limits > 0.0 || rapl(1).is_some() {
        let peak = rapl(1).or(rapl(0)).unwrap_or(0.0) + gpu_limits;
        d.add(
            "What the computer does report",
            "CPU + graphics peak limits",
            format!("about {peak:.0} W (drives, fans and the board add more)"),
        );
    }
    Component {
        id: "sys:psu".into(),
        kind: "psu",
        name: "Power supply".into(),
        subtitle: Some("Not reported by hardware".into()),
        details: d.finish(),
    }
}

// ------------------------------------------------------- deep (own page) ---

/// Extra, longer detail for a component's own page.
pub fn deep(d: &mut Details, id: &str) {
    if id == "board" {
        let pci = Pci::load();
        for p in &pci.0 {
            d.add(
                "PCI devices",
                &p.slot,
                format!("{} · {}", pci_model(p), pci_class_name(p.class)),
            );
        }
    } else if id == "cpu" {
        let info = cpuinfo();
        if let Some(flags) = info.get("flags") {
            d.add("All CPU flags", format!("{} flags", flags.split_whitespace().count()), flags.clone());
        }
        if let Ok(rd) = std::fs::read_dir("/sys/devices/system/cpu/vulnerabilities") {
            let mut rows: Vec<(String, String)> = rd
                .flatten()
                .filter_map(|e| Some((e.file_name().to_string_lossy().into_owned(), read(e.path())?)))
                .collect();
            rows.sort();
            for (k, v) in rows {
                d.add("Security mitigations", k.replace('_', " "), v);
            }
        }
        let mut lines = Vec::new();
        for i in 0..256 {
            match read(format!("/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_cur_freq")).and_then(|v| v.parse::<f64>().ok()) {
                Some(khz) => lines.push(format!("cpu{i:<3} {:.0} MHz", khz / 1000.0)),
                None => break,
            }
        }
        if !lines.is_empty() {
            d.add("Per-thread clock now", "Threads", lines.join("\n"));
        }
    } else if id == "ram" {
        let m = meminfo();
        let mut rows: Vec<_> = m.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        for (k, v) in rows.into_iter().take(14) {
            d.add("Kernel memory counters", k.clone(), format_bytes(*v));
        }
    } else if let Some(disk) = id.strip_prefix("disk:") {
        let udev = run("udevadm", &["info", "--query=property", &format!("--name=/dev/{disk}")]).unwrap_or_default();
        for line in udev.lines().filter(|l| l.starts_with("ID_")) {
            if let Some((k, v)) = line.split_once('=') {
                d.add("Device properties", k.trim_start_matches("ID_"), v);
            }
        }
        let q = |f: &str| read(format!("/sys/block/{disk}/queue/{f}"));
        d.add_opt("I/O queue", "Scheduler", q("scheduler"));
        d.add_opt("I/O queue", "Queue depth", read(format!("/sys/block/{disk}/device/queue_depth")));
        d.add_opt("I/O queue", "Read-ahead", q("read_ahead_kb").map(|v| format!("{v} KiB")));
        d.add_opt("I/O queue", "Max request", q("max_sectors_kb").map(|v| format!("{v} KiB")));
    } else if id.starts_with("gpu:") {
        if let Some(out) = run("nvidia-smi", &["-q", "-d", "CLOCK,POWER,TEMPERATURE,PERFORMANCE"]) {
            let mut section = String::from("Driver report");
            for line in out.lines().skip(1) {
                let t = line.trim();
                if t.is_empty() || t.starts_with("GPU ") {
                    continue;
                }
                match t.split_once(':') {
                    Some((k, v)) if !v.trim().is_empty() => d.add(section.clone(), k.trim(), v.trim()),
                    Some((k, _)) => section = format!("Driver report · {}", k.trim()),
                    None => {}
                }
            }
        }
    }
}
