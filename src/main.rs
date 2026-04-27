//! Chauve, a notification system.
mod entities;
mod notifications;
mod poller;
mod providers;
mod register;
mod settings;
mod tray;
mod ui;

use crate::{register::icon_bytes, ui::app::App};
use dioxus::{
    desktop::{WindowBuilder, tao::window::Icon},
    prelude::*,
};
use tracing::Level;
use tray_icon::dpi::LogicalSize;

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("logger failed to init");
    register::register_app();

    launch(App);
}

pub fn load_icon() -> Icon {
    let image = image::load_from_memory(icon_bytes())
        .expect("Failed to load icon")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

fn launch(app: fn() -> Element) {
    let window =  WindowBuilder::new()
        .with_title("Chauve")
        .with_visible(true)
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .with_min_inner_size(LogicalSize::new(400.0, 300.0))
        .with_max_inner_size(LogicalSize::new(1920.0, 1080.0))
        // .with_resizable(true)
        // .with_maximized(false)
        // .with_fullscreen(None)
        // .with_decorations(false)  // Window borders
        // .with_transparent(false)
        // .with_always_on_top(false)
        // .with_window_icon(Some(load_icon()))
        // .with_min_inner_size(LogicalSize::new(800, 600))
        // .with_closable(false)
        ;
    LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                // .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::WindowHides)
                .with_icon(load_icon())
                .with_window(window),
        )
        .launch(app);
}
