use crate::entities::alert::Alert;
use crate::entities::settings::Settings;
use crate::poller::Poller;
use crate::settings::get_settings;
use dioxus::prelude::*;

#[component]
fn LastUpdate(last_poll_time: String, on_settings_toggle: EventHandler) -> Element {
    let mut poller_signal: Signal<Poller> = use_context();
    let mut settings_signal: Signal<Settings> = use_context();

    rsx! {
        "last poll time: {last_poll_time}"

        div { class: "fixed top-0 right-0 flex gap-1 p-1",
            button {
                onclick: move |_| {
                    // Re-read settings from disk and rebuild the poller.
                    let new_settings = get_settings();
                    *poller_signal.write() = Poller::new(new_settings.clone());
                    *settings_signal.write() = new_settings;

                    let poller = poller_signal.read().clone();
                    spawn(async move {
                        if let Ok(data) = poller.poll_once().await {
                            let mut write_poller = poller_signal.write();
                            write_poller.update_with(data);
                        }
                    });
                },
                id: "update",
                class: "bg-orange-500 hover:bg-orange-700 text-black py-2 px-4 rounded",
                "Update!"
            }
            button {
                onclick: move |_| on_settings_toggle.call(()),
                class: "bg-orange-500 hover:bg-orange-700 text-black py-2 px-4 rounded",
                "⚙"
            }
        }
    }
}

#[component]
pub fn AlertDetails(alert: Alert, unpin: EventHandler) -> Element {
    rsx!{
        div {
            class: "scroll-auto whitespace-pre-wrap",
            onclick: move |_| unpin.call(()),
            "{alert.description}"
        }

    }
}

#[component]
pub fn Header(
    active_alert: Option<Alert>,
    is_pinned: bool,
    last_poll_time: String,
    unpin: EventHandler,
    on_settings_toggle: EventHandler,
) -> Element {
    rsx! {
        div { class: "fixed top-0 right-0 left-0 overflow-y-scroll h-12 p-1",
            match &active_alert {
                Some(alert) => rsx! {
                    AlertDetails { alert: alert.clone(), unpin }
                },
                None => rsx! {
                    LastUpdate { last_poll_time, on_settings_toggle }
                },
            }
        }
    }
}
