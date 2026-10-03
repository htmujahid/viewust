use nusb::DeviceInfo;

pub(crate) const HID: u8 = 0x03;

pub(crate) fn classify(device: &DeviceInfo) -> Vec<&'static str> {
    let name = format!(
        "{} {}",
        device.manufacturer_string().unwrap_or_default(),
        device.product_string().unwrap_or_default()
    )
    .to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));

    let mut classes: Vec<(u8, u8)> = device
        .interfaces()
        .map(|i| (i.class(), i.protocol()))
        .collect();
    classes.push((device.class(), 0));
    let class = |c: u8| classes.iter().any(|(class, _)| *class == c);
    let hid_protocol = |p: u8| classes.iter().any(|(c, proto)| *c == HID && *proto == p);

    let mut kinds: Vec<&'static str> = Vec::new();
    let mut add = |kind: &'static str, found: bool| {
        if found && !kinds.contains(&kind) {
            kinds.push(kind);
        }
    };

    let webcam = has(&["webcam", "camera", "c920", "c922", "brio"]) || class(0x0e);
    add("webcam", webcam);
    let padded = format!(" {name} ");
    let microphone = has(&["microphone", "yeti", "snowball", "podcast", "condenser"])
        || padded.contains(" mic ");
    add("microphone", microphone);
    add(
        "audio",
        !webcam
            && !microphone
            && (has(&[
                "headset",
                "headphone",
                "earbud",
                "airpods",
                "speaker",
                "dac",
            ]) || class(0x01)),
    );
    add(
        "gamepad",
        has(&[
            "gamepad",
            "controller",
            "joystick",
            "xbox",
            "dualshock",
            "dualsense",
        ]),
    );
    add("keyboard", has(&["keyboard"]) || hid_protocol(1));
    add(
        "mouse",
        has(&["mouse", "trackball", "touchpad", "trackpad"]) || hid_protocol(2),
    );
    add("printer", class(0x07) || has(&["printer"]));
    add(
        "storage",
        class(0x08) || has(&["flash", "storage", "ssd", "disk", "card reader"]),
    );
    add(
        "securitykey",
        class(0x0b) || has(&["smart card", "smartcard", "yubikey"]),
    );
    add(
        "phone",
        class(0x06) || has(&["iphone", "ipad", "android", "pixel", "galaxy", "phone"]),
    );

    let says_keyboard = has(&["keyboard"]);
    let says_mouse = has(&["mouse", "trackball"]);
    if says_keyboard && !says_mouse {
        kinds.retain(|k| *k != "mouse");
    } else if says_mouse && !says_keyboard {
        kinds.retain(|k| *k != "keyboard");
    }

    if kinds.is_empty() {
        let fallback = if class(0xe0)
            || has(&[
                "bluetooth",
                "wireless",
                "wi-fi",
                "wlan",
                "dongle",
                "receiver",
            ]) {
            "wireless"
        } else if class(0x09) {
            "hub"
        } else {
            "generic"
        };
        kinds.push(fallback);
    }
    kinds
}
