use std::collections::BTreeSet;
use std::path::Path;

use super::model::{NetInterface, OsNetwork};
use crate::common::cmd::run;
use crate::common::sysfs::read;
use crate::common::Details;

pub(crate) struct RawInterface {
    pub(crate) name: String,
    pub(crate) state: String,
    pub(crate) link_type: String,
    pub(crate) mac: Option<String>,
    pub(crate) mtu: Option<u64>,
    pub(crate) ipv4: Vec<String>,
    pub(crate) ipv6: usize,
}

/// `ip -j addr`: one object per interface, addresses under `addr_info`.
pub(crate) fn parse_ip_addr(json: &str) -> Vec<RawInterface> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    root.as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| {
            let addrs = i["addr_info"].as_array().cloned().unwrap_or_default();
            Some(RawInterface {
                name: i["ifname"].as_str()?.to_owned(),
                state: i["operstate"].as_str().unwrap_or("UNKNOWN").to_owned(),
                link_type: i["link_type"].as_str().unwrap_or("").to_owned(),
                mac: i["address"]
                    .as_str()
                    .map(str::to_owned)
                    .filter(|m| m != "00:00:00:00:00:00"),
                mtu: i["mtu"].as_u64(),
                ipv4: addrs
                    .iter()
                    .filter(|a| a["family"] == "inet")
                    .filter_map(|a| {
                        Some(format!(
                            "{}/{}",
                            a["local"].as_str()?,
                            a["prefixlen"].as_u64()?
                        ))
                    })
                    .collect(),
                ipv6: addrs.iter().filter(|a| a["family"] == "inet6").count(),
            })
        })
        .collect()
}

/// `/proc/net/tcp`: hex local addresses, state 0A is listening, 01 established.
pub(crate) fn parse_sockets(text: &str) -> (BTreeSet<u16>, usize) {
    let mut listening = BTreeSet::new();
    let mut established = 0;
    for line in text.lines().skip(1) {
        let mut f = line.split_whitespace();
        let (Some(_), Some(local), Some(_), Some(state)) = (f.next(), f.next(), f.next(), f.next())
        else {
            continue;
        };
        match state {
            "0A" => {
                if let Some(port) = local
                    .rsplit(':')
                    .next()
                    .and_then(|p| u16::from_str_radix(p, 16).ok())
                {
                    listening.insert(port);
                }
            }
            "01" => established += 1,
            _ => {}
        }
    }
    (listening, established)
}

pub(crate) fn parse_resolv(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("nameserver"))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn kind_of(raw: &RawInterface) -> &'static str {
    let sys = |f: &str| Path::new(&format!("/sys/class/net/{}/{f}", raw.name)).exists();
    if raw.link_type == "loopback" {
        "loopback"
    } else if sys("wireless") || sys("phy80211") {
        "wifi"
    } else if sys("bridge") {
        "bridge"
    } else if !sys("device") {
        "virtual"
    } else {
        "ethernet"
    }
}

fn stat(name: &str, which: &str) -> u64 {
    read(format!("/sys/class/net/{name}/statistics/{which}"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

pub(crate) fn snapshot() -> OsNetwork {
    let raw = run("ip", &["-j", "addr"])
        .map(|t| parse_ip_addr(&t))
        .unwrap_or_default();
    let interfaces: Vec<NetInterface> = raw
        .iter()
        .map(|r| NetInterface {
            kind: kind_of(r),
            speed: read(format!("/sys/class/net/{}/speed", r.name))
                .filter(|s| s != "-1")
                .map(|s| format!("{s} Mb/s")),
            rx: stat(&r.name, "rx_bytes"),
            tx: stat(&r.name, "tx_bytes"),
            name: r.name.clone(),
            state: r.state.clone(),
            mac: r.mac.clone(),
            mtu: r.mtu,
            ipv4: r.ipv4.clone(),
            ipv6: r.ipv6,
        })
        .collect();
    let up = interfaces
        .iter()
        .filter(|i| i.state == "UP" && i.kind != "loopback")
        .count();

    let route = run("ip", &["-j", "route", "show", "default"])
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| {
            let r = v.as_array()?.first()?.clone();
            Some(format!(
                "via {} on {}",
                r["gateway"].as_str().unwrap_or("?"),
                r["dev"].as_str().unwrap_or("?")
            ))
        });

    // The stub at 127.0.0.53 is systemd-resolved; the real servers hide behind it.
    let mut dns = std::fs::read_to_string("/etc/resolv.conf")
        .map(|t| parse_resolv(&t))
        .unwrap_or_default();
    let resolved = dns.iter().any(|s| s == "127.0.0.53");
    if resolved {
        if let Ok(real) = std::fs::read_to_string("/run/systemd/resolve/resolv.conf") {
            let upstream = parse_resolv(&real);
            if !upstream.is_empty() {
                dns = upstream;
            }
        }
    }

    let sockets = |path: &str| {
        std::fs::read_to_string(path)
            .map(|t| parse_sockets(&t))
            .unwrap_or_default()
    };
    let (tcp4, est4) = sockets("/proc/net/tcp");
    let (tcp6, est6) = sockets("/proc/net/tcp6");
    let listening_tcp: Vec<u16> = tcp4.union(&tcp6).copied().collect();
    let (udp4, _) = sockets("/proc/net/udp");
    let (udp6, _) = sockets("/proc/net/udp6");

    let mut d = Details::new();
    d.add_opt("Identity", "Host name", read("/proc/sys/kernel/hostname"));
    d.add_opt("Reaching the internet", "Default route", route.clone());
    d.add(
        "Reaching the internet",
        "DNS",
        if dns.is_empty() {
            "No servers found".to_owned()
        } else {
            format!(
                "{}{}",
                dns.join(", "),
                if resolved {
                    " · through systemd-resolved"
                } else {
                    ""
                }
            )
        },
    );
    d.add(
        "Open ports",
        "TCP, listening",
        if listening_tcp.is_empty() {
            "None".to_owned()
        } else {
            listening_tcp
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        },
    );
    d.add(
        "Open ports",
        "Established connections",
        (est4 + est6).to_string(),
    );
    d.add(
        "Open ports",
        "UDP sockets",
        (udp4.len() + udp6.len()).to_string(),
    );
    OsNetwork {
        up,
        total: interfaces.iter().filter(|i| i.kind != "loopback").count(),
        default_route: route,
        dns,
        listening_tcp,
        established: est4 + est6,
        interfaces,
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IP_ADDR: &str = r#"[
      {"ifname":"lo","operstate":"UNKNOWN","link_type":"loopback","address":"00:00:00:00:00:00","mtu":65536,
       "addr_info":[{"family":"inet","local":"127.0.0.1","prefixlen":8},{"family":"inet6","local":"::1","prefixlen":128}]},
      {"ifname":"enp3s0","operstate":"UP","link_type":"ether","address":"34:5a:60:57:b4:90","mtu":1500,
       "addr_info":[{"family":"inet","local":"192.168.1.23","prefixlen":24},{"family":"inet6","local":"fe80::1","prefixlen":64}]}]"#;

    #[test]
    fn interfaces_give_their_state_mac_and_addresses() {
        let ifs = parse_ip_addr(IP_ADDR);
        assert_eq!(ifs.len(), 2);
        assert_eq!(ifs[0].mac, None, "the all-zero loopback MAC says nothing");
        assert_eq!(ifs[1].state, "UP");
        assert_eq!(ifs[1].ipv4, ["192.168.1.23/24"]);
        assert_eq!(ifs[1].ipv6, 1);
        assert_eq!(ifs[1].mtu, Some(1500));
        assert!(parse_ip_addr("not json").is_empty());
    }

    #[test]
    fn listening_ports_and_established_connections_are_read_from_proc() {
        let tcp = "\
  sl  local_address rem_address   st tx_queue
   0: 0100007F:058C 00000000:0000 0A 00000000:00000000
   1: 1700A8C0:AE42 5DB8D9AC:01BB 01 00000000:00000000
   2: 00000000:0016 00000000:0000 0A 00000000:00000000
   3: 0100007F:058C 00000000:0000 0A 00000000:00000000
";
        let (listening, established) = parse_sockets(tcp);
        assert_eq!(listening.into_iter().collect::<Vec<_>>(), [22, 1420]);
        assert_eq!(established, 1);
    }

    #[test]
    fn nameservers_are_picked_out_of_resolv_conf() {
        let r = "# comment\nnameserver 127.0.0.53\noptions edns0\nsearch lan\nnameserver 1.1.1.1\n";
        assert_eq!(parse_resolv(r), ["127.0.0.53", "1.1.1.1"]);
    }

    #[test]
    fn this_machine_describes_its_network() {
        let n = snapshot();
        assert!(!n.interfaces.is_empty());
        assert!(!n.details.is_empty());
    }
}
