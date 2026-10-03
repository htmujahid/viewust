pub(crate) struct Endpoint {
    pub(crate) interface: u8,
    pub(crate) alt: u8,
    pub(crate) address: u8,
    pub(crate) attributes: u8,
    pub(crate) max_packet: u16,
    pub(crate) interval: u8,
}

#[derive(Default)]
pub(crate) struct Descriptors {
    pub(crate) config_attributes: Option<u8>,
    pub(crate) endpoints: Vec<Endpoint>,
    pub(crate) hid: std::collections::HashMap<u8, (u16, u8, u16)>,
}

pub(crate) fn parse_descriptors(raw: &[u8]) -> Descriptors {
    let mut out = Descriptors::default();
    let (mut interface, mut alt) = (0u8, 0u8);
    let mut o = 0;
    while o + 2 <= raw.len() {
        let len = raw[o] as usize;
        if len < 2 || o + len > raw.len() {
            break;
        }
        let b = &raw[o..o + len];
        match b[1] {
            0x02 if len >= 8 => out.config_attributes = Some(b[7]),
            0x04 if len >= 9 => {
                interface = b[2];
                alt = b[3];
            }
            0x05 if len >= 7 => out.endpoints.push(Endpoint {
                interface,
                alt,
                address: b[2],
                attributes: b[3],
                max_packet: u16::from_le_bytes([b[4], b[5]]) & 0x7ff,
                interval: b[6],
            }),
            0x21 if len >= 9 => {
                let version = u16::from_le_bytes([b[2], b[3]]);
                let report_len = u16::from_le_bytes([b[7], b[8]]);
                out.hid.insert(interface, (version, b[4], report_len));
            }
            _ => {}
        }
        o += len;
    }
    out
}

pub(crate) fn endpoint_text(e: &Endpoint, speed_mbps: f64) -> String {
    let kind = ["Control", "Isochronous", "Bulk", "Interrupt"][(e.attributes & 3) as usize];
    let dir = if e.address & 0x80 != 0 {
        "IN (to computer)"
    } else {
        "OUT (to device)"
    };
    let mut text = format!("{dir} · {kind} · {} bytes/packet", e.max_packet);
    if matches!(e.attributes & 3, 1 | 3) && e.interval > 0 {
        let micros = if speed_mbps >= 480.0 {
            125.0 * 2f64.powi(e.interval as i32 - 1)
        } else if e.attributes & 3 == 1 {
            1000.0 * 2f64.powi(e.interval as i32 - 1)
        } else {
            1000.0 * e.interval as f64
        };
        text += &format!(
            " · polled every {} ({:.0} Hz)",
            fmt_micros(micros),
            1_000_000.0 / micros
        );
    }
    text
}

fn fmt_micros(us: f64) -> String {
    if us >= 1000.0 {
        format!("{} ms", us / 1000.0)
    } else {
        format!("{us} µs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mouse() -> Vec<u8> {
        let mut d = vec![9, 2, 34, 0, 1, 1, 0, 0xA0, 50];
        d.extend([9, 4, 1, 0, 1, 3, 1, 2, 0]);
        d.extend([9, 0x21, 0x11, 0x01, 0, 1, 0x22, 214, 0]);
        d.extend([7, 5, 0x82, 0x03, 8, 0, 4]);
        d
    }

    #[test]
    fn walks_the_descriptor_chain() {
        let d = parse_descriptors(&mouse());
        assert_eq!(d.config_attributes, Some(0xA0));
        assert_eq!(d.hid[&1], (0x0111, 0, 214));
        assert_eq!(d.endpoints.len(), 1);
        let e = &d.endpoints[0];
        assert_eq!(
            (e.interface, e.alt, e.address, e.max_packet, e.interval),
            (1, 0, 0x82, 8, 4)
        );
    }

    #[test]
    fn polling_rate_depends_on_bus_speed() {
        let e = &parse_descriptors(&mouse()).endpoints[0];
        assert_eq!(
            endpoint_text(e, 12.0),
            "IN (to computer) · Interrupt · 8 bytes/packet · polled every 4 ms (250 Hz)"
        );
        assert!(endpoint_text(e, 480.0).contains("polled every 1 ms (1000 Hz)"));
    }

    #[test]
    fn survives_truncated_input() {
        assert!(parse_descriptors(&[9, 2, 34]).endpoints.is_empty());
        assert!(parse_descriptors(&[]).endpoints.is_empty());
    }
}
