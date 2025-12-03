use super::alert::AlertList;
use crate::poller::Poller;
use crate::ui::header::Header;
use crate::settings::get_settings;
use tokio::time::Duration;
use dioxus::prelude::*;
use tokio::time::interval;

// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn AlertsApp() -> Element {
    let mut poller_signal: Signal<Poller> = use_context();
    let mut alerts = use_resource(move || async move {
        let mut poller = poller_signal.write();
        poller.poll().await
    });

    // poll regularly

    use_coroutine::<(), _, _>(move |_| async move {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;
            alerts.restart();
        }
    });

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS}
        document::Stylesheet { href: MAIN_CSS}
        Header { }
        AlertList { }

        div { class: "flex justify-center",
            button {
                onclick: move |_| alerts.restart(),
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
        AlertsApp { }
    }
}
