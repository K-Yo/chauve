//! Chauve, a notification system.
mod entities;
mod notifications;
mod ui;
mod providers;
mod settings;
mod poller;
mod register;

use crate::ui::app::App;
use tracing::Level;

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("logger failed to init");
    register::register_app();
    dioxus::launch(App);
}
