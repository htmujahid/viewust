use super::model::Peripheral;
use crate::common::{Detail, Details};

#[cfg(target_os = "linux")]
pub(crate) fn audio_jacks() -> Vec<Peripheral> {
    use std::process::Command;

    let Ok(cards) = std::fs::read_to_string("/proc/asound/cards") else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for line in cards.lines() {
        let Some((index, rest)) = line.trim_start().split_once(' ') else {
            continue;
        };
        let Ok(index) = index.parse::<u32>() else {
            continue;
        };
        if std::path::Path::new(&format!("/proc/asound/card{index}/usbid")).exists() {
            continue;
        }
        let card_name = rest.rsplit_once(" - ").map(|(_, n)| n.trim().to_owned());

        let Ok(output) = Command::new("amixer")
            .args(["-c", &index.to_string(), "contents"])
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&output.stdout);

        let mut current: Option<String> = None;
        for line in text.lines() {
            if line.starts_with("numid=") {
                current = line
                    .contains("iface=CARD")
                    .then(|| line.split("name='").nth(1))
                    .flatten()
                    .and_then(|n| n.strip_suffix('\''))
                    .filter(|n| n.ends_with(" Jack"))
                    .map(str::to_owned);
            } else if line.trim() == ": values=on" {
                if let Some(jack) = current.take() {
                    out.extend(jack_device(index, &jack, card_name.clone()));
                }
            }
        }
    }
    out
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn audio_jacks() -> Vec<Peripheral> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn jack_device(card: u32, jack: &str, card_name: Option<String>) -> Option<Peripheral> {
    let label = jack.trim_end_matches(" Jack");
    let lower = label.to_lowercase();
    let (kind, name) = if lower.contains("mic") {
        (
            "microphone",
            format!("{label} microphone").replace(" Mic microphone", " microphone"),
        )
    } else if lower.contains("headphone") {
        ("audio", format!("{label}s"))
    } else if lower.contains("line out") || lower.contains("speaker") {
        ("audio", format!("{label} speakers"))
    } else {
        return None;
    };
    let details = jack_details(card, jack, card_name.as_deref());
    Some(Peripheral {
        id: format!("jack-{card}-{}", lower.replace(' ', "-")),
        via: None,
        wireless: false,
        name,
        manufacturer: card_name,
        kind,
        connection: "3.5 mm jack".into(),
        vendor_id: String::new(),
        product_id: String::new(),
        serial_number: None,
        details,
    })
}

#[cfg(target_os = "linux")]
fn jack_details(card: u32, jack: &str, card_name: Option<&str>) -> Vec<Detail> {
    let mut d = Details::new();
    d.add("Connection", "Type", "3.5 mm analog jack");
    d.add("Connection", "Jack", jack.trim_end_matches(" Jack"));
    d.add("Connection", "State", "Something is plugged in");
    d.add_opt("Sound card", "Card", card_name);
    d.add("Sound card", "Card number", card.to_string());
    let codec = std::fs::read_to_string(format!("/proc/asound/card{card}/codec#0"))
        .ok()
        .and_then(|t| {
            t.lines()
                .find_map(|l| l.strip_prefix("Codec:").map(|c| c.trim().to_owned()))
        });
    d.add_opt("Sound card", "Audio codec", codec);
    d.finish()
}
