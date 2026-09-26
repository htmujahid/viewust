//! External monitors.

use super::edid;
use super::model::Display;
use tauri::window::Monitor;

/// The laptop's own panel is part of the computer, not an attached device.
fn is_built_in(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with("edp") || n.starts_with("lvds") || n.starts_with("dsi") || n.contains("built-in")
}

/// External monitors, each matched to its EDID identity where one can be read.
pub(crate) fn list(monitors: Vec<Monitor>, primary: Option<Monitor>) -> Vec<Display> {
    let mut edids = edid::connected_edids();

    monitors
        .into_iter()
        .filter(|m| !m.name().is_some_and(|n| is_built_in(n)))
        .map(|m| {
            let name = m.name().cloned().unwrap_or_else(|| "Display".into());
            let is_primary = primary
                .as_ref()
                .is_some_and(|p| p.position() == m.position() && p.size() == m.size());

            // Toolkits name a monitor by connector ("DP-3") or by model;
            // match either, otherwise hand out the next unclaimed one.
            let found = edids
                .iter()
                .position(|(connector, e)| *connector == name || e.model() == Some(name.as_str()))
                .or_else(|| (!edids.is_empty()).then_some(0));
            let (connector, edid) = match found {
                Some(i) => {
                    let (c, e) = edids.remove(i);
                    (Some(c), Some(e))
                }
                None => (None, None),
            };

            let details = edid::display_details(&edid::DisplayFacts {
                width: m.size().width,
                height: m.size().height,
                x: m.position().x,
                y: m.position().y,
                scale_factor: m.scale_factor(),
                primary: is_primary,
                connector: connector.as_deref(),
                edid: edid.as_ref(),
            });
            Display {
                connector: connector.clone(),
                width_cm: edid.as_ref().and_then(|e| e.width_cm()),
                name: edid
                    .as_ref()
                    .and_then(|e| e.model())
                    .map(str::to_owned)
                    .unwrap_or(name),
                width: m.size().width,
                height: m.size().height,
                scale_factor: m.scale_factor(),
                primary: is_primary,
                details,
            }
        })
        .collect()
}
