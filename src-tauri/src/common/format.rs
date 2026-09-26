//! Human-readable numbers.

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(crate) fn decimal_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1e12 {
        format!("{:.1} TB", b / 1e12)
    } else if b >= 1e9 {
        format!("{:.0} GB", b / 1e9)
    } else {
        format!("{:.0} MB", b / 1e6)
    }
}

pub(crate) fn watts(microwatts: u64) -> String {
    format!("{:.0} W", microwatts as f64 / 1e6)
}

pub(crate) fn secs(ticks: u64) -> String {
    let s = ticks as f64 / 100.0; // clock ticks are 100 Hz on Linux
    if s >= 3600.0 {
        format!(
            "{}h {}m {:.0}s",
            (s / 3600.0) as u64,
            ((s % 3600.0) / 60.0) as u64,
            s % 60.0
        )
    } else if s >= 60.0 {
        format!("{}m {:.0}s", (s / 60.0) as u64, s % 60.0)
    } else {
        format!("{s:.2} s")
    }
}

pub(crate) fn duration(total: u64) -> String {
    let (d, h, m) = (total / 86400, (total % 86400) / 3600, (total % 3600) / 60);
    match (d, h) {
        (0, 0) => format!("{m}m {}s", total % 60),
        (0, _) => format!("{h}h {m}m"),
        _ => format!("{d}d {h}h {m}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_use_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(1 << 30), "1.0 GiB");
    }

    #[test]
    fn drive_sizes_use_decimal_units() {
        assert_eq!(decimal_size(1_000_204_886_016), "1.0 TB");
        assert_eq!(decimal_size(500_107_862_016), "500 GB");
        assert_eq!(decimal_size(538_000_000), "538 MB");
    }

    #[test]
    fn cpu_time_reads_naturally() {
        assert_eq!(secs(150), "1.50 s");
        assert_eq!(secs(6000), "1m 0s");
        assert_eq!(secs(360_000), "1h 0m 0s");
    }

    #[test]
    fn durations_drop_empty_leading_units() {
        assert_eq!(duration(59), "0m 59s");
        assert_eq!(duration(3700), "1h 1m");
        assert_eq!(duration(90_060), "1d 1h 1m");
    }
}
