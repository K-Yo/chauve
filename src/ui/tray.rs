use dioxus::desktop::trayicon::menu::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{Icon, TrayIcon, TrayIconBuilder};
use dioxus::desktop::{icon_from_memory, use_muda_event_handler};
use dioxus::prelude::*;
use image::Rgba;
use tracing::debug;

use crate::entities::alert::Alert;
use crate::register::icon_bytes;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new();
    let show_item: MenuItem = MenuItem::new("Show", true, None);
    let quit_item: MenuItem = MenuItem::new("Quit", true, None);
    menu.append_items(&[&show_item, &PredefinedMenuItem::separator(), &quit_item])?;

    let icon = icon_from_memory(icon_bytes()).expect("valid icon");

    let builder = TrayIconBuilder::new()
        .with_menu_on_left_click(false)
        .with_menu(Box::new(menu))
        .with_tooltip("Chauve")
        .with_icon(icon);

    let tray_icon: Signal<TrayIcon> =
        use_signal(|| builder.build().expect("tray icon builder failed"));
    provide_context(tray_icon);
    {
        use_muda_event_handler(move |event| match &event.id {
            id if id == quit_item.id() => {
                info!("Exiting");
                std::process::exit(0);
            }
            id if id == show_item.id() => {
                debug!("showing");
                let service = dioxus::desktop::window();
                let window = &service.window;
                window.set_visible(true);
                window.set_minimized(false);
                window.set_focus();
            }
            _ => {}
        });
    }
    Ok(())
}

/// Counts alerts by severity level.
///
/// Iterates over the provided alerts and tallies how many fall into each
/// of the four known severity buckets: critical, high, medium, and low.
/// Any severity that doesn't match one of those names is ignored.
///
/// The returned vector is always ordered from most to least severe:
/// `critical`, `high`, `medium`, `low`.
///
/// # Arguments
///
/// * `alerts` — A slice of [`Alert`] instances to classify.
///
/// # Returns
///
/// A `Vec<(&'static str, usize)>` where each pair is `(severity_name, count)`.
fn count_by_severity(alerts: &[Alert]) -> Vec<(&'static str, usize)> {
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for alert in alerts {
        match alert.severity.machinename.as_str() {
            "critical" => critical += 1,
            "high" => high += 1,
            "medium" => medium += 1,
            "low" => low += 1,
            _ => {}
        }
    }

    vec![
        ("critical", critical),
        ("high", high),
        ("medium", medium),
        ("low", low),
    ]
}

/// Generates a tray icon with colored corner badges indicating active alerts by severity.
///
/// The base icon is overlaid with up to four solid square badges, one in each corner:
/// - **Top-left**: critical (`#780000`)
/// - **Top-right**: high (`#c1121f`)
/// - **Bottom-left**: medium (`#fb8500`)
/// - **Bottom-right**: low (`#ffc300`)
///
/// A badge is drawn only when there is at least one alert for that severity. When no alerts
/// are present the original icon is returned unchanged.
pub fn generate_severity_icon(alerts: &[Alert]) -> Icon {
    let counts = count_by_severity(alerts);
    let has_any = counts.iter().any(|(_, c)| *c > 0);

    if !has_any {
        return icon_from_memory(icon_bytes()).expect("valid icon");
    }

    let base_img = image::load_from_memory(icon_bytes())
        .expect("valid icon")
        .to_rgba8();
    let (width, height) = (base_img.width(), base_img.height());
    let mut img = base_img;

    let badge_size = (width.min(height) / 4).max(4);

    let colors = [
        Rgba([0x78, 0x00, 0x00, 0xFF]), // critical
        Rgba([0xC1, 0x12, 0x1F, 0xFF]), // high
        Rgba([0xFB, 0x85, 0x00, 0xFF]), // medium
        Rgba([0xFF, 0xC3, 0x00, 0xFF]), // low
    ];

    let corners = [
        (0u32, 0u32),                              // top-left
        (width.saturating_sub(badge_size), 0u32),  // top-right
        (0u32, height.saturating_sub(badge_size)), // bottom-left
        (
            width.saturating_sub(badge_size),
            height.saturating_sub(badge_size),
        ), // bottom-right
    ];

    for i in 0..4 {
        let (_, count) = counts[i];
        if count == 0 {
            continue;
        }
        let color = colors[i];
        let (cx, cy) = corners[i];

        for dy in 0..badge_size {
            for dx in 0..badge_size {
                let x = cx + dx;
                let y = cy + dy;
                if x >= width || y >= height {
                    continue;
                }
                img.put_pixel(x, y, color);
            }
        }
    }

    let rgba = img.into_raw();
    Icon::from_rgba(rgba, width, height).expect("valid icon")
}
