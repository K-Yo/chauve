use crate::entities::alert::{Alert, format_age};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;

#[component]
pub fn AlertComponent(
    alert: Alert,
    is_pinned: bool,
    now: DateTime<Utc>,
    on_click: EventHandler<String>,
    on_mouse_enter: EventHandler<String>,
    on_mouse_leave: EventHandler<String>,
) -> Element {
    let id = alert.id.clone();
    // How long the alert has been firing, blank when the provider gives no start time.
    let age = alert
        .starts_at
        .map(|starts_at| format_age(starts_at, now))
        .unwrap_or_default();
    let starts_at_title = alert
        .starts_at
        .map(|starts_at| starts_at.to_rfc3339())
        .unwrap_or_default();
    let pinned_class = if is_pinned {
        "inset-ring-2 inset-ring-black"
    } else {
        ""
    };
    let severity_class = format!("severity-{}", alert.severity.machinename);
    rsx! {
        div {
            class: "alert flex whitespace-nowrap {severity_class} {pinned_class}",
            onclick: {
                let id = id.clone();
                move |_| on_click.call(id.clone())
            },
            onmouseenter: {
                let id = id.clone();
                move |_| on_mouse_enter.call(id.clone())
            },
            onmouseleave: {
                let id = id.clone();
                move |_| on_mouse_leave.call(id.clone())
            },
            div { class: "p-1 flex-none",
                a { href: alert.link, target: "_blank", "🔍" }
            }
            div {
                class: "p-1 flex-none text-gray-300 text-sm tabular-nums w-16 text-right",
                title: "{starts_at_title}",
                "{age}"
            }
            div { class: "p-1 flex-none font-semibold", "{alert.title}" }
            div { class: "p-1 flex-none text-gray-300 text-sm", "{alert.instance}" }

            div { class: "p-1 flex-1 min-w-0 truncate", "{alert.summary}" }
        }
    }
}

#[component]
pub fn AlertList(
    alerts: Vec<Alert>,
    pinned_id: Option<String>,
    on_click: EventHandler<String>,
    on_mouse_enter: EventHandler<String>,
    on_mouse_leave: EventHandler<String>,
) -> Element {
    // Sort alerts by severity, with critical first
    let mut sorted_alerts = alerts.clone();
    sorted_alerts.sort_by_key(|alert| alert.severity.order());

    // One reference instant for the whole list so every row agrees on "now".
    let now = Utc::now();

    rsx! {
        div { class: "overflow-x-scroll overflow-y-scroll pt-12 pb-6",
            div { class: "min-w-full",
                for alert in sorted_alerts {
                    AlertComponent {
                        alert: alert.clone(),
                        is_pinned: pinned_id == Some(alert.id),
                        now,
                        on_click,
                        on_mouse_enter,
                        on_mouse_leave,
                    }
                }
            }
        }
    }
}
