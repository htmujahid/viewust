//! System tray icon whose menu doubles as a mini monitor: a few read-only lines refreshed every
//! couple of seconds. Linux tray hosts only reliably support menus, so the readout lives there.

use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager};

use crate::common::format::format_bytes;
use crate::monitor::{MonitorService, Sample};

const REFRESH: Duration = Duration::from_secs(2);

pub fn setup(app: &App) -> tauri::Result<()> {
    let line = |id: &str, text: &str| MenuItem::with_id(app, id, text, false, None::<&str>);
    let cpu = line("cpu", "CPU …")?;
    let memory = line("memory", "Memory …")?;
    let gpu = line("gpu", "GPU …")?;
    let disk = line("disk", "Disk …")?;
    let network = line("network", "Network …")?;
    let battery = line("battery", "Battery …")?;
    let show = MenuItem::with_id(app, "show", "Show Viewust", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let rule = PredefinedMenuItem::separator(app)?;
    let rule2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &cpu, &memory, &gpu, &disk, &network, &battery, &rule, &show, &rule2, &quit,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("monitor")
        .tooltip("Viewust")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    // The tray samples on its own: the page's samples are rates since *its* last call.
    let service = MonitorService::default();
    std::thread::spawn(move || loop {
        let sample = service.sample();
        let lines = summarize(&sample);
        let _ = cpu.set_text(&lines.cpu);
        let _ = memory.set_text(&lines.memory);
        let _ = gpu.set_text(&lines.gpu);
        let _ = disk.set_text(&lines.disk);
        let _ = network.set_text(&lines.network);
        let _ = battery.set_text(&lines.battery);
        std::thread::sleep(REFRESH);
    });
    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

struct Lines {
    cpu: String,
    memory: String,
    gpu: String,
    disk: String,
    network: String,
    battery: String,
}

fn rate(bps: f64) -> String {
    format!("{}/s", format_bytes(bps.max(0.0) as u64))
}

fn summarize(s: &Sample) -> Lines {
    let mut cpu = format!("CPU  {:.0}%", s.cpu.total);
    if let Some(t) = s.cpu.temperature {
        cpu.push_str(&format!(" · {t:.0} °C"));
    }

    let memory = format!(
        "Memory  {} / {}",
        format_bytes(s.memory.used),
        format_bytes(s.memory.total)
    );

    let gpu = match s.gpus.first() {
        Some(g) => match g.util {
            Some(u) => format!("GPU  {u:.0}%"),
            None => "GPU  —".into(),
        },
        None => "GPU  none".into(),
    };

    let disk = match s.volumes.iter().find(|v| v.mount == "/") {
        Some(v) => format!("Disk  {} / {}", format_bytes(v.used), format_bytes(v.total)),
        None => "Disk  —".into(),
    };

    let network = match s.net.first() {
        Some(n) => format!("Net  ↓ {}  ↑ {}", rate(n.rx_bps), rate(n.tx_bps)),
        None => "Net  offline".into(),
    };

    let battery = match s.power.batteries.first() {
        Some(b) => format!("Battery  {:.0}% · {}", b.percent, b.status),
        None => "Battery  none".into(),
    };

    Lines {
        cpu,
        memory,
        gpu,
        disk,
        network,
        battery,
    }
}
