pub mod commands;
mod computer;
mod descriptors;
mod display;
mod driver;
mod hid;
mod input;
mod jack;
mod pci;
mod usb;

use crate::common::{Detail, Details};

pub fn build(id: &str) -> Vec<Detail> {
    let mut d = Details::new();
    #[cfg(target_os = "linux")]
    {
        if id == "computer" {
            computer::report(&mut d);
        } else if let Some(connector) = id.strip_prefix("display:") {
            display::report(&mut d, connector);
        } else if id.starts_with("net:") {
        } else if let Some(rest) = id.strip_prefix("sys:") {
            crate::system::deep::report(&mut d, rest);
        } else if let Some(rest) = id.strip_prefix("jack-") {
            jack::report(&mut d, rest);
        } else {
            usb::report(&mut d, id);
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = id;
        d.add(
            "Technical",
            "Availability",
            "Low-level details are only available on Linux for now",
        );
    }
    d.finish()
}
