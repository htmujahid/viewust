mod dns;
mod route;
mod wifi;

use serde::Serialize;

use crate::common::cmd::run;
use crate::common::sysfs::*;
use crate::common::{Detail, Details};
use dns::dns_servers;
pub(crate) use route::default_route;
use wifi::{band, channel, is_wifi, nmcli_wifi, parse_iw_link, quality_word, signal_percent};

#[derive(Serialize)]
pub struct Connection {
    kind: &'static str,
    interface: String,
    link_label: String,
    signal: Option<u32>,
    router_name: String,
    router_details: Vec<Detail>,
    connectivity: &'static str,
    internet_details: Vec<Detail>,
}

pub fn connection() -> Option<Connection> {
    let route = default_route()?;
    let interface = route.interface.clone();
    let wifi = is_wifi(&interface);
    let kind = if wifi {
        "wifi"
    } else if interface.starts_with("tun")
        || interface.starts_with("wg")
        || interface.starts_with("ppp")
        || interface.starts_with("wwan")
    {
        "other"
    } else {
        "ethernet"
    };

    let sys = format!("/sys/class/net/{interface}");
    let speed = read(format!("{sys}/speed"))
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|s| *s > 0);

    let w = if wifi {
        run("iw", &["dev", &interface, "link"])
            .map(|t| parse_iw_link(&t))
            .filter(|w| w.ssid.is_some() || w.signal_dbm.is_some())
    } else {
        None
    };
    let nm_wifi = if wifi && w.is_none() {
        nmcli_wifi()
    } else {
        None
    };

    let signal = w
        .as_ref()
        .and_then(|w| w.signal_dbm)
        .map(signal_percent)
        .or(nm_wifi.as_ref().map(|n| n.1));
    let ssid = w
        .as_ref()
        .and_then(|w| w.ssid.clone())
        .or(nm_wifi.as_ref().map(|n| n.0.clone()));
    let freq = w
        .as_ref()
        .and_then(|w| w.frequency_mhz)
        .or(nm_wifi.as_ref().map(|n| n.2));

    let link_label = if wifi {
        [
            Some("Wi-Fi".to_owned()),
            freq.map(|f| band(f).to_owned()),
            w.as_ref()
                .and_then(|w| w.rx_rate.clone())
                .map(|r| r.replace("MBit/s", "Mbit/s")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ")
    } else if let Some(s) = speed {
        format!(
            "Ethernet · {}",
            if s >= 1000 {
                format!("{} Gbit/s", s as f64 / 1000.0)
            } else {
                format!("{s} Mbit/s")
            }
        )
    } else {
        match kind {
            "other" => "Tunnel / mobile link".to_owned(),
            _ => "Ethernet".to_owned(),
        }
    };

    let gateway_mac = read("/proc/net/arp").and_then(|t| {
        t.lines().skip(1).find_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            (f.first() == Some(&route.gateway.as_str()))
                .then(|| f.get(3).map(|m| m.to_string()))
                .flatten()
        })
    });

    let dns = run("resolvectl", &["status", &interface])
        .map(|t| dns_servers(&t))
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| {
            read("/etc/resolv.conf")
                .map(|t| {
                    t.lines()
                        .filter_map(|l| l.strip_prefix("nameserver "))
                        .map(|s| s.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default()
        });

    let connectivity = match run("nmcli", &["-t", "-g", "CONNECTIVITY", "general"])
        .as_deref()
        .map(str::trim)
    {
        Some("full") => "full",
        Some("limited") => "limited",
        Some("portal") => "portal",
        Some("none") => "none",
        _ => "unknown",
    };

    let mut r = Details::new();
    r.add("Router", "Address (gateway)", &route.gateway);
    r.add_opt("Router", "Hardware address", gateway_mac);
    r.add(
        "Router",
        "Reached through",
        if wifi {
            ssid.clone()
                .map(|s| format!("Wi-Fi network “{s}”"))
                .unwrap_or_else(|| "Wi-Fi".into())
        } else {
            format!("{} on {interface}", link_label)
        },
    );
    if !dns.is_empty() {
        r.add("Router", "DNS servers", dns.join(", "));
    }
    if wifi {
        r.add_opt("Wi-Fi", "Network name", ssid.clone());
        r.add_opt(
            "Wi-Fi",
            "Access point",
            w.as_ref().and_then(|w| w.bssid.clone()),
        );
        r.add_opt(
            "Wi-Fi",
            "Signal",
            w.as_ref()
                .and_then(|w| w.signal_dbm)
                .map(|d| format!("{d} dBm · {}", quality_word(signal_percent(d))))
                .or(nm_wifi
                    .as_ref()
                    .map(|n| format!("{}% · {}", n.1, quality_word(n.1)))),
        );
        r.add_opt(
            "Wi-Fi",
            "Band",
            freq.map(|f| format!("{} · {f} MHz", band(f))),
        );
        r.add_opt(
            "Wi-Fi",
            "Channel",
            freq.and_then(channel).map(|c| c.to_string()),
        );
        r.add_opt(
            "Wi-Fi",
            "Receive rate",
            w.as_ref().and_then(|w| w.rx_rate.clone()),
        );
        r.add_opt(
            "Wi-Fi",
            "Send rate",
            w.as_ref().and_then(|w| w.tx_rate.clone()),
        );
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
    i.add(
        "Internet",
        "Via",
        format!("{interface} → {}", route.gateway),
    );
    i.add(
        "Internet",
        "Public address",
        "Not checked: Viewust makes no outside connections",
    );
    let ips: Vec<String> = sysinfo::Networks::new_with_refreshed_list()
        .iter()
        .filter(|(n, _)| **n == interface)
        .flat_map(|(_, d)| {
            d.ip_networks()
                .iter()
                .map(|ip| format!("{}/{}", ip.addr, ip.prefix))
                .collect::<Vec<_>>()
        })
        .collect();
    i.add("Your address", "On this network", ips.join(", "));
    i.add_opt(
        "Your address",
        "Hardware address",
        read(format!("{sys}/address")),
    );
    i.add_opt("Your address", "MTU", read(format!("{sys}/mtu")));
    let total = |f: &str| {
        read(format!("{sys}/statistics/{f}"))
            .and_then(|v| v.parse::<u64>().ok())
            .map(crate::common::format::format_bytes)
    };
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
