//! Chauve, a notification system.
mod entities;
mod notifications;
mod poller;
mod providers;
mod register;
mod settings;
mod ui;

mod tray;

use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use crate::tray::TrayCommand;
use crate::ui::app::App;
use tracing::Level;

/// Global channel receiver so the Dioxus coroutine can consume tray events
/// without needing to thread the receiver through `dioxus::launch`.
pub static TRAY_RX: std::sync::OnceLock<Mutex<Receiver<TrayCommand>>> = std::sync::OnceLock::new();

fn init_tray() {
    match tray::Tray::new() {
        Ok((tray, rx)) => {
            // Keep `tray` alive for the entire process.
            std::mem::forget(tray);
            TRAY_RX.set(Mutex::new(rx)).ok();
        }
        Err(e) => {
            tracing::warn!("System tray unavailable: {e}");
        }
    }
}

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("logger failed to init");
    register::register_app();
    // Tray must be created before the Dioxus event loop starts (macOS
    // requires tray operations on the main thread, and they must exist before
    // the window is shown).
    init_tray();

    dioxus::launch(App);
}
