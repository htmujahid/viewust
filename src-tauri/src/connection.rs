//! How this computer reaches the internet: wired or Wi-Fi, through which
//! router, with what signal. Read from the routing table, NetworkManager and
//! `iw`. Nothing here sends traffic anywhere; "online" means a route exists
//! and, when NetworkManager is present, what it already knows.

use crate::detail::{Detail, Details};
use serde::Serialize;
use std::process::Command;

pub struct Route {
    pub interface: String,
    pub gateway: String,
}

/// The lowest-metric default route in /proc/net/route.
pub fn default_route() -> Option<Route> {
    let text = std::fs::read_to_string("/proc/net/route").ok()?;
    let mut best: Option<(u32, Route)> = None;
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 8 || f[1] != "00000000" {
            continue;
        }
        let flags = u32::from_str_radix(f[3], 16).unwrap_or(0);
        if flags & 0b11 != 0b11 {
            continue; // needs to be both up and a gateway route
        }
        let metric = f[6].parse::<u32>().unwrap_or(0);
        if best.as_ref().map_or(true, |(m, _)| metric < *m) {
            best = Some((metric, Route { interface: f[0].to_owned(), gateway: hex_ip(f[2])? }));
        }
    }
    best.map(|(_, r)| r)
}

/// "0101A8C0" (little-endian hex) → "192.168.1.1".
fn hex_ip(hex: &str) -> Option<String> {
    let v = u32::from_str_radix(hex, 16).ok()?;
    let b = v.to_le_bytes();
    Some(format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]))
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read(path: impl AsRef<std::path::Path>) -> Option<String> {
    crate::detail::read(path)
}

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
            w.rx_rate = v.split_whitespace().take(2).collect::<Vec<_>>().join(" ").into();
        } else if let Some(v) = line.strip_prefix("tx bitrate:") {
            w.tx_rate = v.split_whitespace().take(2).collect::<Vec<_>>().join(" ").into();
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

fn band(mhz: u32) -> &'static str {
    match mhz {
        0..=2500 => "2.4 GHz",
        2501..=5900 => "5 GHz",
        _ => "6 GHz",
    }
}

fn is_wifi(interface: &str) -> bool {
    let base = format!("/sys/class/net/{interface}");
    std::path::Path::new(&format!("{base}/wireless")).exists() || std::path::Path::new(&format!("{base}/phy80211")).exists()
}

#[derive(Serialize)]
pub struct Connection {
    /// "ethernet", "wifi" or "other" (VPN, mobile, …).
    kind: &'static str,
    interface: String,
    /// Short text for the link between the computer and the router.
    link_label: String,
    /// 0–100 for Wi-Fi, otherwise null.
    signal: Option<u32>,
    router_name: String,
    router_details: Vec<Detail>,
    /// "full", "limited", "portal", "none" or "unknown" (as NetworkManager sees it).
    connectivity: &'static str,
    internet_details: Vec<Detail>,
}

pub fn connection() -> Option<Connection> {
    let route = default_route()?;
    let interface = route.interface.clone();
    let wifi = is_wifi(&interface);
    let kind = if wifi {
        "wifi"
    } else if interface.starts_with("tun") || interface.starts_with("wg") || interface.starts_with("ppp") || interface.starts_with("wwan") {
        "other"
    } else {
        "ethernet"
    };

    let sys = format!("/sys/class/net/{interface}");
    let speed = read(format!("{sys}/speed")).and_then(|v| v.parse::<i64>().ok()).filter(|s| *s > 0);

    let w = if wifi {
        run("iw", &["dev", &interface, "link"]).map(|t| parse_iw_link(&t)).filter(|w| w.ssid.is_some() || w.signal_dbm.is_some())
    } else {
        None
    };
    // `iw` may be missing; NetworkManager knows the network name and strength too.
    let nm_wifi = if wifi && w.is_none() { nmcli_wifi() } else { None };

    let signal = w.as_ref().and_then(|w| w.signal_dbm).map(signal_percent).or(nm_wifi.as_ref().map(|n| n.1));
    let ssid = w.as_ref().and_then(|w| w.ssid.clone()).or(nm_wifi.as_ref().map(|n| n.0.clone()));
    let freq = w.as_ref().and_then(|w| w.frequency_mhz).or(nm_wifi.as_ref().map(|n| n.2));

    let link_label = if wifi {
        [
            Some("Wi-Fi".to_owned()),
            freq.map(|f| band(f).to_owned()),
            w.as_ref().and_then(|w| w.rx_rate.clone()).map(|r| r.replace("MBit/s", "Mbit/s")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ")
    } else if let Some(s) = speed {
        format!("Ethernet · {}", if s >= 1000 { format!("{} Gbit/s", s as f64 / 1000.0) } else { format!("{s} Mbit/s") })
    } else {
        match kind {
            "other" => "Tunnel / mobile link".to_owned(),
            _ => "Ethernet".to_owned(),
        }
    };

    // the router's hardware address, from the neighbour table
    let gateway_mac = read("/proc/net/arp").and_then(|t| {
        t.lines().skip(1).find_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            (f.first() == Some(&route.gateway.as_str())).then(|| f.get(3).map(|m| m.to_string())).flatten()
        })
    });

    let dns = run("resolvectl", &["status", &interface]).map(|t| dns_servers(&t)).filter(|d| !d.is_empty()).unwrap_or_else(|| {
        read("/etc/resolv.conf")
            .map(|t| t.lines().filter_map(|l| l.strip_prefix("nameserver ")).map(|s| s.trim().to_owned()).collect())
            .unwrap_or_default()
    });

    let connectivity = match run("nmcli", &["-t", "-g", "CONNECTIVITY", "general"]).as_deref().map(str::trim) {
        Some("full") => "full",
        Some("limited") => "limited",
        Some("portal") => "portal",
        Some("none") => "none",
        _ => "unknown",
    };

    let mut r = Details::new();
    r.add("Router", "Address (gateway)", &route.gateway);
    r.add_opt("Router", "Hardware address", gateway_mac);
    r.add("Router", "Reached through", if wifi { ssid.clone().map(|s| format!("Wi-Fi network “{s}”")).unwrap_or_else(|| "Wi-Fi".into()) } else { format!("{} on {interface}", link_label) });
    if !dns.is_empty() {
        r.add("Router", "DNS servers", dns.join(", "));
    }
    if wifi {
        r.add_opt("Wi-Fi", "Network name", ssid.clone());
        r.add_opt("Wi-Fi", "Access point", w.as_ref().and_then(|w| w.bssid.clone()));
        r.add_opt("Wi-Fi", "Signal", w.as_ref().and_then(|w| w.signal_dbm).map(|d| format!("{d} dBm · {}", quality_word(signal_percent(d)))).or(nm_wifi.as_ref().map(|n| format!("{}% · {}", n.1, quality_word(n.1)))));
        r.add_opt("Wi-Fi", "Band", freq.map(|f| format!("{} · {f} MHz", band(f))));
        r.add_opt("Wi-Fi", "Channel", freq.and_then(channel).map(|c| c.to_string()));
        r.add_opt("Wi-Fi", "Receive rate", w.as_ref().and_then(|w| w.rx_rate.clone()));
        r.add_opt("Wi-Fi", "Send rate", w.as_ref().and_then(|w| w.tx_rate.clone()));
    } else {
        r.add_opt("Cable", "Link speed", speed.map(|s| format!("{s} Mbit/s")));
        r.add_opt("Cable", "Duplex", read(format!("{sys}/duplex")));
    }

    let mut i = Details::new();
    i.add(
        "Internet",
        "Status",
        match connectivity {
            "full" => "Online (confirmed by NetworkManager)",
            "limited" => "Limited: connected to the network but no internet",
            "portal" => "Sign-in required (captive portal)",
            "none" => "Not connected",
            _ => "A route to the internet exists",
        },
    );
    i.add("Internet", "Via", format!("{interface} → {}", route.gateway));
    i.add(
        "Internet",
        "Public address",
        "Not checked: Viewust makes no outside connections",
    );
    let ips: Vec<String> = sysinfo::Networks::new_with_refreshed_list()
        .iter()
        .filter(|(n, _)| **n == interface)
        .flat_map(|(_, d)| d.ip_networks().iter().map(|ip| format!("{}/{}", ip.addr, ip.prefix)).collect::<Vec<_>>())
        .collect();
    i.add("Your address", "On this network", ips.join(", "));
    i.add_opt("Your address", "Hardware address", read(format!("{sys}/address")));
    i.add_opt("Your address", "MTU", read(format!("{sys}/mtu")));
    let total = |f: &str| read(format!("{sys}/statistics/{f}")).and_then(|v| v.parse::<u64>().ok()).map(crate::detail::format_bytes);
    i.add_opt("Since startup", "Downloaded", total("rx_bytes"));
    i.add_opt("Since startup", "Uploaded", total("tx_bytes"));

    Some(Connection {
        kind,
        interface,
        link_label,
        signal,
        router_name: ssid.unwrap_or_else(|| "Router".into()),
        router_details: r.finish(),
        connectivity,
        internet_details: i.finish(),
    })
}

fn quality_word(percent: u32) -> &'static str {
    match percent {
        0..=29 => "weak",
        30..=59 => "fair",
        60..=79 => "good",
        _ => "excellent",
    }
}

/// "DNS Servers: 1.1.1.1 8.8.8.8" (or "Current DNS Server: …") from `resolvectl status`.
pub fn dns_servers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(v) = line.strip_prefix("DNS Servers:").or_else(|| line.strip_prefix("Current DNS Server:")) {
            for s in v.split_whitespace() {
                if !out.iter().any(|x| x == s) {
                    out.push(s.to_owned());
                }
            }
        }
    }
    out
}

/// (network name, strength 0–100, frequency MHz) of the active Wi-Fi network from nmcli.
fn nmcli_wifi() -> Option<(String, u32, u32)> {
    let text = run("nmcli", &["-t", "-f", "IN-USE,SSID,SIGNAL,FREQ", "dev", "wifi"])?;
    parse_nmcli_wifi(&text)
}

pub fn parse_nmcli_wifi(text: &str) -> Option<(String, u32, u32)> {
    text.lines().find(|l| l.starts_with('*')).and_then(|l| {
        // terse mode escapes colons inside values as "\:"
        let parts: Vec<String> = l.replace("\\:", "\u{1}").split(':').map(|p| p.replace('\u{1}', ":")).collect();
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
    fn hex_ip_is_little_endian() {
        assert_eq!(hex_ip("0101A8C0").as_deref(), Some("192.168.1.1"));
        assert_eq!(hex_ip("010011AC").as_deref(), Some("172.17.0.1"));
    }

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
    fn parses_nmcli_and_dns() {
        let nm = " :Cafe:70:2412 MHz\n*:My\\:Wifi:64:5220 MHz\n";
        assert_eq!(parse_nmcli_wifi(nm), Some(("My:Wifi".into(), 64, 5220)));
        assert_eq!(dns_servers("Link 3 (wlan0)\n  Current DNS Server: 1.1.1.1\n         DNS Servers: 1.1.1.1 8.8.8.8\n"), vec!["1.1.1.1", "8.8.8.8"]);
    }
}
