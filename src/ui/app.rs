use super::alert::AlertList;
use crate::poller::Poller;
use crate::settings::get_settings;
use crate::ui::header::Header;
use dioxus::prelude::*;
use tokio::time::Duration;
use tokio::time::interval;

// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
#[component]
pub fn AlertsApp() -> Element {
    let mut poller_signal: Signal<Poller> = use_context();

    // poll regularly

    use_coroutine::<(), _, _>(move |_| async move {
        // Get settings to determine poll frequency
        let settings = get_settings();
        debug!("{:#?}",settings);
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

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
        Header {}
        AlertList {}

        div { class: "flex justify-center",
            button {
                onclick: move |_| {
                    // Same logic as above
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
