pub struct Route {
    pub interface: String,
    pub gateway: String,
}

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
            continue;
        }
        let metric = f[6].parse::<u32>().unwrap_or(0);
        if best.as_ref().map_or(true, |(m, _)| metric < *m) {
            best = Some((
                metric,
                Route {
                    interface: f[0].to_owned(),
                    gateway: hex_ip(f[2])?,
                },
            ));
        }
    }
    best.map(|(_, r)| r)
}

fn hex_ip(hex: &str) -> Option<String> {
    let v = u32::from_str_radix(hex, 16).ok()?;
    let b = v.to_le_bytes();
    Some(format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]))
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
    fn rejects_garbage() {
        assert_eq!(hex_ip("not hex"), None);
    }
}
