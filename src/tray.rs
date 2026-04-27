use dioxus::desktop::trayicon::TrayIconBuilder;
use dioxus::desktop::trayicon::menu::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::{icon_from_memory, use_muda_event_handler, use_tray_menu_event_handler};
use dioxus::prelude::*;
use tracing::debug;

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

    provide_context(builder.build().expect("tray icon builder failed"));
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
