use super::alert::AlertList;
use crate::poller::PollStats;
use crate::poller::Poller;
use crate::poller::PollerData;
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
    let mut fetcher: Resource<Result<PollerData, PollStats>> =
        use_resource(move || async move {
            let poller = poller_signal.read(); // immutable borrow only — safe!
            let fetched_data = poller.poll_once().await?;

            // Only now: acquire write lock briefly to update
            drop(poller); // explicit: drop read

            let mut write_poller = poller_signal.write();
            write_poller.update_with(fetched_data.clone());
            Ok(fetched_data)
        });

    // poll regularly

    use_coroutine::<(), _, _>(move |_| async move {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;
            fetcher.restart();
        }
    });

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
        Header {}
        AlertList {}

        div { class: "flex justify-center",
            button {
                onclick: move |_| fetcher.restart(),
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
