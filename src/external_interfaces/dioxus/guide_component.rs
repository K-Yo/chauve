use crate::alert::Alert;
use crate::grafana::GrafanaAlert;
use crate::transfo::convert_response;
use dioxus::prelude::*;
// const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const FONT: Asset = asset!("/assets/fonts/FiraMonoNerdFont/FiraMonoNerdFont-Regular.otf");

use tokio::time::{interval, Duration};


#[component]
pub fn AlertsApp() -> Element {
    dbg!(FONT);
    let mut alerts = use_resource(|| async move {
        let client = reqwest::Client::new();
        let response = client
            .get("http://172.31.31.240:3000/api/alertmanager/grafana/api/v2/alerts")
            .bearer_auth("glsa_wc9nHEy0gI0dvbG8Rr2CL3eDke6kFRkK_999d0778")
            .send()
            .await
            .unwrap();
        let api_alerts = response.json::<ApiResponse>().await.unwrap();
        convert_response(api_alerts)
    });

    // poll regularly

    use_coroutine::<(),_,_>(move |_| async move {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;
            alerts.restart();
        };
    });

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS}
        document::Stylesheet { href: MAIN_CSS}
        for alert in alerts.cloned().unwrap_or_default() {
        AlertComponent { alert }
        }

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
pub fn AlertComponent(alert: Alert) -> Element {
    let severity_class = format!("severity-{}", alert.severity);
    rsx! {
        div { class: "alert {severity_class}",
            "{alert.name}"
        }
    }
}
