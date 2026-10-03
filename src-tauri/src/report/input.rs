pub(crate) fn bitmap(text: &str) -> Vec<usize> {
    let width = usize::BITS as usize;
    let mut out = Vec::new();
    for (i, word) in text.split_whitespace().rev().enumerate() {
        if let Ok(v) = u64::from_str_radix(word, 16) {
            for b in 0..width.min(64) {
                if v >> b & 1 == 1 {
                    out.push(i * width + b);
                }
            }
        }
    }
    out
}

pub(crate) fn named(
    bits: &[usize],
    name: impl Fn(usize) -> Option<&'static str>,
) -> Vec<&'static str> {
    bits.iter().filter_map(|b| name(*b)).collect()
}

pub(crate) fn ev_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "Sync",
        0x01 => "Keys / buttons",
        0x02 => "Relative motion",
        0x03 => "Absolute position",
        0x04 => "Scan codes",
        0x05 => "Switches",
        0x11 => "LEDs",
        0x12 => "Sound",
        0x14 => "Key repeat",
        0x15 => "Force feedback",
        _ => return None,
    })
}

pub(crate) fn rel_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "X",
        0x01 => "Y",
        0x02 => "Z",
        0x06 => "Horizontal wheel",
        0x07 => "Dial",
        0x08 => "Wheel",
        0x09 => "Misc",
        0x0b => "Wheel (high resolution)",
        0x0c => "Horizontal wheel (high resolution)",
        _ => return None,
    })
}

pub(crate) fn abs_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "X",
        0x01 => "Y",
        0x02 => "Z",
        0x03 => "Rx",
        0x04 => "Ry",
        0x05 => "Rz",
        0x10 => "Hat 0",
        0x18 => "Pressure",
        0x2f => "Multitouch slot",
        0x35 => "Touch X",
        0x36 => "Touch Y",
        _ => return None,
    })
}

pub(crate) fn button_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x110 => "Left",
        0x111 => "Right",
        0x112 => "Middle",
        0x113 => "Side",
        0x114 => "Extra",
        0x115 => "Forward",
        0x116 => "Back",
        0x117 => "Task",
        0x130 => "A / South",
        0x131 => "B / East",
        0x133 => "X / North",
        0x134 => "Y / West",
        0x136 => "Left bumper",
        0x137 => "Right bumper",
        0x13a => "Select",
        0x13b => "Start",
        0x13c => "Mode",
        0x13d => "Left stick",
        0x13e => "Right stick",
        _ => return None,
    })
}

pub(crate) fn led_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0 => "Num Lock",
        1 => "Caps Lock",
        2 => "Scroll Lock",
        3 => "Compose",
        4 => "Kana",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmaps_list_set_bits_from_the_least_significant_word() {
        assert_eq!(bitmap("1f0000 0 0 0 0"), vec![272, 273, 274, 275, 276]);
        assert_eq!(bitmap("0"), Vec::<usize>::new());
        assert_eq!(bitmap("garbage"), Vec::<usize>::new());
    }

    #[test]
    fn button_codes_have_names() {
        let names = named(&bitmap("1f0000 0 0 0 0"), button_name);
        assert_eq!(names, vec!["Left", "Right", "Middle", "Side", "Extra"]);
        assert_eq!(
            named(&[0, 1, 6, 8], rel_name),
            vec!["X", "Y", "Horizontal wheel", "Wheel"]
        );
    }
}
