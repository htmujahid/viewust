use serde::Serialize;

use crate::common::cmd::run_any;

const HOST: &str = "https://speed.cloudflare.com";

#[derive(Serialize, Debug)]
pub struct SpeedTest {
    /// Time to first byte of a tiny request, the best of three
    pub(crate) latency_ms: Option<f64>,
    pub(crate) download_bps: Option<f64>,
    pub(crate) upload_bps: Option<f64>,
    pub(crate) server: &'static str,
}

/// curl's `-w` figures come back as plain decimals; zero means it never got going.
pub(crate) fn parse_rate(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().filter(|v| *v > 0.0)
}

pub(crate) fn seconds_to_ms(text: &str) -> Option<f64> {
    parse_rate(text).map(|s| s * 1000.0)
}

/// One transfer's verdict: `-w "%{http_code} %{size_download} %{time_total}"`. Anything but
/// success is thrown away, so an error page can never read as a speed.
pub(crate) fn parse_probe(text: &str) -> Option<(u64, f64)> {
    let mut f = text.split_whitespace();
    let code: u16 = f.next()?.parse().ok()?;
    let bytes: f64 = f.next()?.parse().ok()?;
    let seconds: f64 = f.next()?.parse().ok()?;
    ((200..300).contains(&code) && bytes > 0.0 && seconds > 0.0).then_some((bytes as u64, seconds))
}

/// A rate out of one or more successful transfers, as total bytes over total time.
pub(crate) fn combined_rate(probes: &[(u64, f64)]) -> Option<f64> {
    let bytes: u64 = probes.iter().map(|(b, _)| b).sum();
    let seconds: f64 = probes.iter().map(|(_, s)| s).sum();
    (bytes > 0 && seconds > 0.0).then(|| bytes as f64 / seconds)
}

fn curl(args: &[&str]) -> Option<String> {
    let mut all = vec!["-s", "-o", "/dev/null"];
    all.extend_from_slice(args);
    run_any("curl", &all)
}

/// The download endpoint caps each request (somewhere under 100 MB), so the test pulls
/// 50 MB chunks until its time budget is gone and sums them.
const CHUNK: &str = "50000000";
const BUDGET: f64 = 10.0;

fn download() -> Option<f64> {
    let mut probes = Vec::new();
    let mut left = BUDGET;
    for _ in 0..8 {
        if left < 1.0 {
            break;
        }
        let max_time = format!("{:.0}", left.ceil());
        let out = curl(&[
            "-w",
            "%{http_code} %{size_download} %{time_total}",
            "--max-time",
            &max_time,
            &format!("{HOST}/__down?bytes={CHUNK}"),
        ])?;
        let Some(probe) = parse_probe(&out) else {
            break;
        };
        left -= probe.1;
        probes.push(probe);
    }
    combined_rate(&probes)
}

/// Measures the line by actually using it: up to a couple of hundred megabytes on a fast
/// connection.
pub(crate) fn run_test() -> SpeedTest {
    // Latency: three tiny requests, keep the quickest start of transfer.
    let latency_ms = (0..3)
        .filter_map(|_| {
            curl(&[
                "-w",
                "%{time_starttransfer}",
                "--max-time",
                "5",
                &format!("{HOST}/__down?bytes=1000"),
            ])
            .as_deref()
            .and_then(seconds_to_ms)
        })
        .min_by(f64::total_cmp);

    let download_bps = download();

    // Upload: a bounded body of zeros, so the test can't run away.
    let upload = std::env::temp_dir().join(format!("viewust-upload-{}", std::process::id()));
    let upload_bps = std::fs::write(&upload, vec![0u8; 16 * 1024 * 1024])
        .ok()
        .and_then(|_| {
            curl(&[
                "-w",
                "%{http_code} %{size_upload} %{time_total}",
                "--max-time",
                "10",
                "-H",
                "Content-Type: application/octet-stream",
                "--data-binary",
                &format!("@{}", upload.display()),
                &format!("{HOST}/__up"),
            ])
        })
        .as_deref()
        .and_then(parse_probe)
        .and_then(|p| combined_rate(&[p]));
    let _ = std::fs::remove_file(&upload);

    SpeedTest {
        latency_ms,
        download_bps,
        upload_bps,
        server: "speed.cloudflare.com",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curl_figures_parse_and_zero_means_nothing_happened() {
        assert_eq!(parse_rate("1856201.000\n"), Some(1_856_201.0));
        assert_eq!(parse_rate("0.000"), None);
        assert_eq!(parse_rate("garbage"), None);
        assert_eq!(seconds_to_ms("0.442134"), Some(442.134));
    }

    #[test]
    fn an_error_page_never_reads_as_a_speed() {
        // Exactly what Cloudflare's 403 looked like: one byte of body, instantly.
        assert_eq!(parse_probe("403 1 0.102"), None);
        assert_eq!(parse_probe("200 14730242 3.000"), Some((14_730_242, 3.0)));
        assert_eq!(parse_probe("200 0 1.0"), None);
        assert_eq!(parse_probe("garbage"), None);
    }

    #[test]
    fn chunks_add_up_into_one_rate() {
        let rate = combined_rate(&[(50_000_000, 4.0), (25_000_000, 2.0)]).unwrap();
        assert!((rate - 12_500_000.0).abs() < 1.0);
        assert_eq!(combined_rate(&[]), None);
    }
}
