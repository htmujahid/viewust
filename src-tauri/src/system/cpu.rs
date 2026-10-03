use super::model::*;
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

pub(crate) fn cpuinfo() -> std::collections::HashMap<String, String> {
    let text = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    text.split("\n\n")
        .next()
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect()
}

pub(crate) fn cpu() -> Component {
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
        match (
            info.get("cpu family"),
            info.get("model"),
            info.get("stepping"),
        ) {
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
        d.add(
            "Clock",
            "Range",
            format!("{:.1} – {:.1} GHz", lo / 1e6, hi / 1e6),
        );
    }
    if !sys.cpus().is_empty() {
        let avg =
            sys.cpus().iter().map(|c| c.frequency()).sum::<u64>() as f64 / sys.cpus().len() as f64;
        d.add("Clock", "Now (average)", format!("{:.2} GHz", avg / 1000.0));
    }
    d.add_opt(
        "Clock",
        "Governor",
        read(cpu0.join("cpufreq/scaling_governor")),
    );
    for i in 0..6 {
        let c = cpu0.join(format!("cache/index{i}"));
        if let (Some(level), Some(kind), Some(size)) = (
            read(c.join("level")),
            read(c.join("type")),
            read(c.join("size")),
        ) {
            d.add("Cache", format!("L{level} {}", kind.to_lowercase()), size);
        }
    }

    let rapl = |n: u8| {
        read(format!(
            "/sys/class/powercap/intel-rapl:0/constraint_{n}_power_limit_uw"
        ))
        .and_then(|v| v.parse::<u64>().ok())
    };
    d.add_opt(
        "Power and heat",
        "Sustained power limit (PL1)",
        rapl(0).map(watts),
    );
    d.add_opt(
        "Power and heat",
        "Boost power limit (PL2)",
        rapl(1).map(watts),
    );
    d.add_opt(
        "Power and heat",
        "Temperature",
        hwmon_temp(&["coretemp"], Some("Package id 0"))
            .or_else(|| hwmon_temp(&["k10temp"], None))
            .map(|t| format!("{t:.0} °C")),
    );

    let flags = info.get("flags").cloned().unwrap_or_default();
    let has = |f: &str| flags.split_whitespace().any(|x| x == f);
    let sets: Vec<&str> = ["sse4_2", "avx", "avx2", "avx512f", "fma", "aes", "sha_ni"]
        .into_iter()
        .filter(|f| has(f))
        .collect();
    d.add(
        "Features",
        "Instruction sets",
        sets.join(", ").to_uppercase(),
    );
    d.add(
        "Features",
        "Virtualization",
        if has("vmx") || has("svm") {
            "Supported"
        } else {
            "Not available"
        },
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
