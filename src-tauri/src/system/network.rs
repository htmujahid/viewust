use super::model::*;
use super::pci::{pci_model, Pci};
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;

pub(crate) fn network(pci: &Pci) -> Vec<Component> {
    let nets = sysinfo::Networks::new_with_refreshed_list();
    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/net")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        let p = e.path();
        if !p.join("device").exists() || name == "lo" {
            continue;
        }
        let wifi = p.join("wireless").exists() || p.join("phy80211").exists();
        let slot = std::fs::canonicalize(p.join("device"))
            .ok()
            .and_then(|r| r.file_name().map(|n| n.to_string_lossy().into_owned()));
        let model = slot.as_deref().and_then(|s| pci.at(s)).map(pci_model);
        let driver = std::fs::read_link(p.join("device/driver"))
            .ok()
            .and_then(|l| l.file_name().map(|n| n.to_string_lossy().into_owned()));

        let mut d = Details::new();
        d.add("Adapter", "Interface", &name);
        d.add("Adapter", "Type", if wifi { "Wi-Fi" } else { "Ethernet" });
        d.add_opt("Adapter", "Model", model.clone());
        d.add_opt(
            "Adapter",
            "Vendor",
            slot.as_deref()
                .and_then(|s| pci.at(s))
                .and_then(|x| x.vendor_name.clone()),
        );
        d.add_opt("Adapter", "Driver", driver);
        d.add_opt("Adapter", "MAC address", read(p.join("address")));
        d.add_opt("Link", "State", read(p.join("operstate")));
        d.add_opt(
            "Link",
            "Speed",
            read(p.join("speed"))
                .and_then(|s| s.parse::<i64>().ok())
                .filter(|s| *s > 0)
                .map(|s| {
                    if s >= 1000 {
                        format!("{} Gbit/s", s as f64 / 1000.0)
                    } else {
                        format!("{s} Mbit/s")
                    }
                }),
        );
        d.add_opt("Link", "Duplex", read(p.join("duplex")));
        d.add_opt("Link", "MTU", read(p.join("mtu")));
        if let Some(data) = nets.iter().find(|(n, _)| **n == name).map(|(_, d)| d) {
            let ips: Vec<String> = data
                .ip_networks()
                .iter()
                .map(|ip| ip.addr.to_string())
                .collect();
            d.add("Addresses", "IP", ips.join(", "));
            d.add("Traffic", "Received", format_bytes(data.total_received()));
            d.add("Traffic", "Sent", format_bytes(data.total_transmitted()));
        }
        out.push(Component {
            id: format!("sys:nic:{name}"),
            kind: "nic",
            name: model.unwrap_or_else(|| name.clone()),
            subtitle: Some(format!(
                "{name} · {}",
                if wifi { "Wi-Fi" } else { "Ethernet" }
            )),
            details: d.finish(),
        });
    }
    out
}
