use crate::entities::alert::format_age;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use tokio::time::Duration;

#[component]
pub fn Footer(last_poll_time: Option<DateTime<Utc>>) -> Element {
    // Ticks every second so the relative age stays current. Local to the footer
    // so the alert list doesn't re-render along with it.
    let mut now: Signal<DateTime<Utc>> = use_signal(Utc::now);
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            now.set(Utc::now());
        }
    });

    let label = match last_poll_time {
        Some(time) => format!("last poll: {} ago", format_age(time, now())),
        None => "last poll: never".to_string(),
    };
    let title = last_poll_time
        .map(|time| time.to_rfc3339())
        .unwrap_or_default();

    rsx! {
        div {
            class: "fixed bottom-0 right-0 left-0 h-6 p-1 text-sm text-gray-300 tabular-nums",
            title: "{title}",
            "{label}"
        }
    }
}
