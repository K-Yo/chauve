use dioxus::prelude::*;
use crate::{entities::Alert, grafana::GrafanaAlert};
type ApiResponse = Vec<GrafanaAlert>;

use self::guide_component::AlertsApp;

fn main() {
    dioxus::launch(App);
    // trpl::run(async { run().await });
}

async fn run() {
    let client = reqwest::Client::new();
    let response = client
        .get("http://172.31.31.240:3000/api/alertmanager/grafana/api/v2/alerts")
        .bearer_auth("glsa_wc9nHEy0gI0dvbG8Rr2CL3eDke6kFRkK_999d0778")
        .send()
        .await
        .unwrap();
    let api_alerts = response.json::<ApiResponse>().await.unwrap();
    println!("{:#?}", api_alerts);
}

#[component]
fn App() -> Element {
    rsx! {
        AlertsApp {}
    }
}
