use super::alert::AlertList;
use crate::poller::Poller;
use crate::settings::get_settings;
use crate::ui::header::Header;
use dioxus::prelude::*;
use tokio::time::Duration;
use tokio::time::interval;

// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn AlertsApp() -> Element {
    let mut poller_signal: Signal<Poller> = use_context();
    // poll regularly

    use_coroutine::<(), _, _>(move |_| async move {
        // Get settings to determine poll frequency
        let settings = get_settings();
        debug!("{:#?}", settings);
        let poll_duration = Duration::from_secs(settings.poll_frequency);
        let mut ticker = interval(poll_duration);

        loop {
            ticker.tick().await;

            // Clone signal to move into async
            let poller = poller_signal.read().clone();
            match poller.poll_once().await {
                Ok(data) => {
                    // Update state
                    let mut write_poller = poller_signal.write();
                    write_poller.update_with(data);
                }
                Err(stats) => {
                    // Optionally log or store error
                    tracing::warn!("Poll failed: {}", stats);
                }
            }
        }
    });

    // Alert selection
    // pinned_id is Some when an alert is expplicitely clicked
    let mut pinned_id: Signal<Option<String>> = use_signal(|| None);
    // hovered_id is Some when an alert is hovered
    let mut hovered_id: Signal<Option<String>> = use_signal(|| None);
    let alerts: Vec<crate::entities::alert::Alert> = poller_signal.read().clone().alerts();
    let last_poll_time = poller_signal.read().clone().last_poll_time();

    let active_alert = {
        let pid = pinned_id();
        let hid = hovered_id();
        let id = pid.or(hid);

        id.and_then(|id| alerts.iter().find(|a| a.id == id).cloned())
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div { class: "h-screen relative",
            Header {
                active_alert,
                is_pinned: pinned_id().is_some(),
                last_poll_time,
                unpin: move |_| pinned_id.set(None),
            }
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

#[component]
pub fn App() -> Element {
    // define app-wide settings
    let settings = get_settings();
    let poller = Poller::new(settings);
    use_context_provider(|| Signal::new(poller));
    rsx! {
        AlertsApp {}
    }
}
