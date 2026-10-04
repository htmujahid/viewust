use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::common::cmd::run;
use crate::common::sysfs::read;

#[derive(Serialize, Debug, PartialEq)]
pub struct AudioDevice {
    pub(crate) name: String,
    pub(crate) volume: u32,
    pub(crate) muted: bool,
    pub(crate) default: bool,
}

#[derive(Serialize, Debug)]
pub struct AudioStream {
    pub(crate) card: String,
    pub(crate) name: String,
    /// "playback" or "capture"
    pub(crate) direction: &'static str,
    pub(crate) rate: Option<u32>,
    pub(crate) channels: Option<u32>,
    pub(crate) format: Option<String>,
    /// The program that owns the stream, when the kernel says
    pub(crate) program: Option<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct AudioCard {
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) driver: String,
}

#[derive(Serialize)]
pub struct AudioSample {
    pub(crate) cards: Vec<AudioCard>,
    pub(crate) sinks: Vec<AudioDevice>,
    pub(crate) sources: Vec<AudioDevice>,
    pub(crate) streams: Vec<AudioStream>,
    pub(crate) playing: usize,
    pub(crate) capturing: usize,
}

/// `/proc/asound/cards`: ` 0 [PCH   ]: HDA-Intel - HDA Intel PCH` and an indented detail line.
pub(crate) fn parse_cards(text: &str) -> Vec<AudioCard> {
    text.lines()
        .filter_map(|line| {
            let (index, rest) = line.trim_start().split_once(' ')?;
            let index: u32 = index.parse().ok()?;
            let (driver, name) = rest.split_once("]:")?.1.trim().split_once(" - ")?;
            Some(AudioCard {
                index,
                name: name.trim().to_owned(),
                driver: driver.trim().to_owned(),
            })
        })
        .collect()
}

/// A device line of `wpctl status`: ` │  *   60. Built-in Audio Analog Stereo [vol: 0.64 MUTED]`.
pub(crate) fn parse_wpctl_device(line: &str) -> Option<AudioDevice> {
    let (head, vol) = line.split_once("[vol:")?;
    let volume = vol
        .split_whitespace()
        .next()?
        .trim_end_matches(']')
        .parse::<f64>()
        .ok()?;
    let (marker, name) = head.split_once(". ")?;
    Some(AudioDevice {
        name: name.trim().to_owned(),
        volume: (volume * 100.0).round() as u32,
        muted: vol.contains("MUTED"),
        default: marker.contains('*'),
    })
}

/// The Sinks/Sources listings inside the Audio part of `wpctl status`.
pub(crate) fn parse_wpctl_status(text: &str) -> (Vec<AudioDevice>, Vec<AudioDevice>) {
    let mut sinks = Vec::new();
    let mut sources = Vec::new();
    let mut bucket: Option<&mut Vec<AudioDevice>> = None;
    for line in text.lines() {
        if line.contains("Sinks:") {
            bucket = Some(&mut sinks);
        } else if line.contains("Sources:") {
            bucket = Some(&mut sources);
        } else if line.contains("Filters:") || line.contains("Streams:") || line.trim() == "Video" {
            bucket = None;
        } else if let (Some(b), Some(d)) = (bucket.as_deref_mut(), parse_wpctl_device(line)) {
            b.push(d);
        }
    }
    (sinks, sources)
}

/// `key: value` lines of a PCM `status` or `hw_params` file.
pub(crate) fn field(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}

fn running_stream(sub: &Path, card_name: &str, direction: &'static str) -> Option<AudioStream> {
    let status = std::fs::read_to_string(sub.join("status")).ok()?;
    if status.trim() == "closed" || field(&status, "state").as_deref() != Some("RUNNING") {
        return None;
    }
    let params = std::fs::read_to_string(sub.join("hw_params")).unwrap_or_default();
    let number =
        |key: &str| field(&params, key).and_then(|v| v.split_whitespace().next()?.parse().ok());
    let program =
        field(&status, "owner_pid").and_then(|pid| read(format!("/proc/{}/comm", pid.trim())));
    Some(AudioStream {
        card: card_name.to_owned(),
        name: sub
            .parent()
            .and_then(|pcm| read(pcm.join("info")).and_then(|t| field(&t, "name")))
            .unwrap_or_else(|| "PCM".into()),
        direction,
        rate: number("rate"),
        channels: number("channels"),
        format: field(&params, "format"),
        program,
    })
}

fn streams_for(cards: &[AudioCard]) -> Vec<AudioStream> {
    let mut out = Vec::new();
    for card in cards {
        let dir = PathBuf::from(format!("/proc/asound/card{}", card.index));
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            let direction = if file.starts_with("pcm") && file.ends_with('p') {
                "playback"
            } else if file.starts_with("pcm") && file.ends_with('c') {
                "capture"
            } else {
                continue;
            };
            for sub in std::fs::read_dir(entry.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                if sub.file_name().to_string_lossy().starts_with("sub") {
                    out.extend(running_stream(&sub.path(), &card.name, direction));
                }
            }
        }
    }
    out
}

pub(crate) fn sample() -> AudioSample {
    let cards = std::fs::read_to_string("/proc/asound/cards")
        .map(|t| parse_cards(&t))
        .unwrap_or_default();
    let streams = streams_for(&cards);
    let (sinks, sources) = run("wpctl", &["status"])
        .map(|t| parse_wpctl_status(&t))
        .unwrap_or_default();
    AudioSample {
        playing: streams.iter().filter(|s| s.direction == "playback").count(),
        capturing: streams.iter().filter(|s| s.direction == "capture").count(),
        cards,
        sinks,
        sources,
        streams,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_give_their_index_driver_and_name() {
        let text = " 0 [PCH            ]: HDA-Intel - HDA Intel PCH\n                      HDA Intel PCH at 0x4012c20000 irq 136\n 1 [C920           ]: USB-Audio - HD Pro Webcam C920\n";
        let c = parse_cards(text);
        assert_eq!(c.len(), 2);
        assert_eq!(
            c[0],
            AudioCard {
                index: 0,
                name: "HDA Intel PCH".into(),
                driver: "HDA-Intel".into()
            }
        );
        assert_eq!(c[1].name, "HD Pro Webcam C920");
    }

    #[test]
    fn wpctl_devices_carry_volume_mute_and_the_default_mark() {
        let d = parse_wpctl_device(" │  *   60. Built-in Audio Analog Stereo        [vol: 0.64]")
            .unwrap();
        assert_eq!(
            d,
            AudioDevice {
                name: "Built-in Audio Analog Stereo".into(),
                volume: 64,
                muted: false,
                default: true
            }
        );
        let m = parse_wpctl_device(" │      38. HDMI Audio [vol: 0.98 MUTED]").unwrap();
        assert!((m.muted, m.default) == (true, false) && m.volume == 98);
        assert!(parse_wpctl_device(" ├─ Devices:").is_none());
    }

    #[test]
    fn status_splits_sinks_from_sources_and_stops_at_video() {
        let text = " ├─ Sinks:\n │  *   60. Out [vol: 0.50]\n │\n ├─ Sources:\n │  *   42. Mic [vol: 0.88]\n │\n ├─ Filters:\n │      99. Echo cancel [vol: 1.00]\n Video\n ├─ Sinks:\n";
        let (sinks, sources) = parse_wpctl_status(text);
        assert_eq!(sinks.len(), 1);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "Mic");
    }

    #[test]
    fn pcm_fields_are_read_by_key() {
        let status = "state: RUNNING\nowner_pid   : 4576\n";
        assert_eq!(field(status, "state").as_deref(), Some("RUNNING"));
        assert_eq!(field(status, "owner_pid").as_deref(), Some("4576"));
        let params = "format: S32_LE\nchannels: 2\nrate: 48000 (48000/1)\n";
        assert_eq!(field(params, "rate").as_deref(), Some("48000 (48000/1)"));
    }

    #[test]
    fn this_machine_reports_its_audio() {
        let a = sample();
        assert!(!a.cards.is_empty());
    }
}
