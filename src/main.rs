//! Chauve, a notification system.
mod entities;
mod ui;
mod providers;
mod settings;
mod poller;
use crate::ui::app::App;
use tracing::Level;

fn main() {
      dioxus_logger::init(Level::DEBUG).expect("logger failed to init");
    dioxus::launch(App);
}
