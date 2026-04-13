use super::alert::AlertList;
use super::settings::SettingsView;
use crate::entities::settings::Settings;
use crate::poller::Poller;
use crate::settings::get_settings;
use crate::ui::header::Header;
use dioxus::prelude::*;
use tokio::time::Duration;

// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: &str = include_str!("../../assets/main.css");

#[component]
pub fn AlertsApp() -> Element {
    let mut poller_signal: Signal<Poller> = use_context();
    let settings_signal: Signal<Settings> = use_context();

    use_coroutine::<(), _, _>(move |_| async move {
        loop {
            // Poll immediately, then sleep for the configured duration.
            // Reading poll_frequency inside the loop means each sleep uses
            // the current value — settings changes take effect on the next cycle.
            let poller = poller_signal.read().clone();
            match poller.poll_once().await {
                Ok(data) => {
                    poller_signal.write().update_with(data);
                }
                Err(stats) => {
                    tracing::warn!("Poll failed: {}", stats);
                }
            }

            let poll_frequency = settings_signal.read().poll_frequency;
            tokio::time::sleep(Duration::from_secs(poll_frequency)).await;
        }
    });

    let mut pinned_id: Signal<Option<String>> = use_signal(|| None);
    let mut hovered_id: Signal<Option<String>> = use_signal(|| None);
    let mut show_settings: Signal<bool> = use_signal(|| false);

    let alerts: Vec<crate::entities::alert::Alert> = poller_signal.read().clone().alerts();
    let last_poll_time = poller_signal.read().clone().last_poll_time();

    let active_alert = {
        let pid = pinned_id();
        let hid = hovered_id();
        let id = pid.or(hid);
        id.and_then(|id| alerts.iter().find(|a| a.id == id).cloned())
    };

    rsx! {
        document::Style { { MAIN_CSS } }
        div { class: "h-screen relative",
            Header {
                active_alert,
                is_pinned: pinned_id().is_some(),
                last_poll_time,
                unpin: move |_| pinned_id.set(None),
                on_settings_toggle: move |_| show_settings.set(!show_settings()),
            }
            if show_settings() {
                SettingsView {}
            } else {
                AlertList {
                    alerts,
                    pinned_id: pinned_id(),
                    on_click: move |id: String| {
                        pinned_id.set(if pinned_id() == Some(id.clone()) { None } else { Some(id) });
                    },
                    on_mouse_enter: move |id: String| hovered_id.set(Some(id)),
                    on_mouse_leave: move |_| hovered_id.set(None),
                }
            }
        }
    }
}

#[component]
pub fn App() -> Element {
    let settings = get_settings();
    // Provide settings first (cloned), then consume original for Poller.
    use_context_provider(|| Signal::new(settings.clone()));
    let poller = Poller::new(settings);
    use_context_provider(|| Signal::new(poller));
    rsx! {
        AlertsApp {}
    }
}
