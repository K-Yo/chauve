use crate::entities::alert::Alert;
use dioxus::prelude::*;

#[component]
pub fn AlertComponent(
    alert: Alert,
    is_pinned: bool,
    on_click: EventHandler<String>,
    on_mouse_enter: EventHandler<String>,
    on_mouse_leave: EventHandler<String>,
) -> Element {
    let id = alert.id.clone();
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
            div { class: "p-1 flex-none", "{alert.title}" }
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

    rsx! {
        div { class: "overflow-x-scroll overflow-y-scroll pt-12",
            div { class: "min-w-full",
                for alert in sorted_alerts {
                    AlertComponent {
                        alert: alert.clone(),
                        is_pinned: pinned_id == Some(alert.id),
                        on_click,
                        on_mouse_enter,
                        on_mouse_leave,
                    }
                }
            }
        }
    }
}
