//! Chauve, a notification system.
mod entities;
mod ui;
mod providers;
mod settings;
mod poller;
use crate::ui::app::App;

fn main() {
    dioxus::launch(App);
}
