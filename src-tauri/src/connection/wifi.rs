//! Wi-Fi link details from `iw` and `nmcli`.

use crate::common::cmd::run;

#[derive(Default, Debug, PartialEq)]
pub struct Wifi {
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub frequency_mhz: Option<u32>,
    pub rx_rate: Option<String>,
    pub tx_rate: Option<String>,
}

/// Parses the output of `iw dev <if> link`.
pub fn parse_iw_link(text: &str) -> Wifi {
    let mut w = Wifi::default();
    for line in text.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("Connected to ") {
            w.bssid = rest.split_whitespace().next().map(str::to_owned);
        } else if let Some(v) = line.strip_prefix("SSID:") {
            w.ssid = Some(v.trim().to_owned());
        } else if let Some(v) = line.strip_prefix("freq:") {
            w.frequency_mhz = v.trim().split('.').next().and_then(|n| n.parse().ok());
        } else if let Some(v) = line.strip_prefix("signal:") {
            w.signal_dbm = v.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if let Some(v) = line.strip_prefix("rx bitrate:") {
            w.rx_rate = v
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
                .into();
        } else if let Some(v) = line.strip_prefix("tx bitrate:") {
            w.tx_rate = v
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
                .into();
        }
    }
    w
}

/// Link quality 0–100 from a signal in dBm (the usual −100 … −50 mapping).
pub fn signal_percent(dbm: i32) -> u32 {
    (2 * (dbm + 100)).clamp(0, 100) as u32
}

pub fn channel(mhz: u32) -> Option<u32> {
    match mhz {
        2412..=2472 => Some((mhz - 2407) / 5),
        2484 => Some(14),
        5000..=5895 => Some((mhz - 5000) / 5),
        5955..=7115 => Some((mhz - 5950) / 5),
        _ => None,
    }
}

pub(crate) fn band(mhz: u32) -> &'static str {
    match mhz {
        0..=2500 => "2.4 GHz",
        2501..=5900 => "5 GHz",
        _ => "6 GHz",
    }
}

pub(crate) fn is_wifi(interface: &str) -> bool {
    let base = format!("/sys/class/net/{interface}");
    std::path::Path::new(&format!("{base}/wireless")).exists()
        || std::path::Path::new(&format!("{base}/phy80211")).exists()
}

pub(crate) fn quality_word(percent: u32) -> &'static str {
    match percent {
        0..=29 => "weak",
        30..=59 => "fair",
        60..=79 => "good",
        _ => "excellent",
    }
}

/// (network name, strength 0–100, frequency MHz) of the active Wi-Fi network from nmcli.
pub(crate) fn nmcli_wifi() -> Option<(String, u32, u32)> {
    let text = run(
        "nmcli",
        &["-t", "-f", "IN-USE,SSID,SIGNAL,FREQ", "dev", "wifi"],
    )?;
    parse_nmcli_wifi(&text)
}

pub fn parse_nmcli_wifi(text: &str) -> Option<(String, u32, u32)> {
    text.lines().find(|l| l.starts_with('*')).and_then(|l| {
        // terse mode escapes colons inside values as "\:"
        let parts: Vec<String> = l
            .replace("\\:", "\u{1}")
            .split(':')
            .map(|p| p.replace('\u{1}', ":"))
            .collect();
        Some((
            parts.get(1)?.clone(),
            parts.get(2)?.parse().ok()?,
            parts.get(3)?.split_whitespace().next()?.parse().ok()?,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iw_link() {
        let sample = "Connected to a4:2b:b0:11:22:33 (on wlan0)\n\tSSID: Home Net\n\tfreq: 5180.0\n\tRX: 1234 bytes (10 packets)\n\tsignal: -52 dBm\n\trx bitrate: 866.7 MBit/s VHT-MCS 9 80MHz short GI VHT-NSS 2\n\ttx bitrate: 650.0 MBit/s VHT-MCS 7\n";
        let w = parse_iw_link(sample);
        assert_eq!(w.ssid.as_deref(), Some("Home Net"));
        assert_eq!(w.bssid.as_deref(), Some("a4:2b:b0:11:22:33"));
        assert_eq!(w.frequency_mhz, Some(5180));
        assert_eq!(w.signal_dbm, Some(-52));
        assert_eq!(w.rx_rate.as_deref(), Some("866.7 MBit/s"));
        assert_eq!(w.tx_rate.as_deref(), Some("650.0 MBit/s"));
    }

    #[test]
    fn signal_and_channel_maps() {
        assert_eq!(signal_percent(-52), 96);
        assert_eq!(signal_percent(-100), 0);
        assert_eq!(signal_percent(-30), 100);
        assert_eq!(channel(2412), Some(1));
        assert_eq!(channel(2437), Some(6));
        assert_eq!(channel(5180), Some(36));
        assert_eq!(channel(5955), Some(1));
        assert_eq!(band(2437), "2.4 GHz");
        assert_eq!(band(5180), "5 GHz");
        assert_eq!(band(5955), "6 GHz");
    }

    #[test]
    fn describes_signal_in_words() {
        assert_eq!(quality_word(10), "weak");
        assert_eq!(quality_word(45), "fair");
        assert_eq!(quality_word(70), "good");
        assert_eq!(quality_word(96), "excellent");
    }

    #[test]
    fn reads_the_active_network_from_nmcli() {
        // terse mode escapes a colon inside a name as "\:"
        let nm = " :Cafe:70:2412 MHz\n*:My\\:Wifi:64:5220 MHz\n";
        assert_eq!(parse_nmcli_wifi(nm), Some(("My:Wifi".into(), 64, 5220)));
        assert_eq!(parse_nmcli_wifi(" :Cafe:70:2412 MHz\n"), None);
    }
}
