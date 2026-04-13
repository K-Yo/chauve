use crate::entities::alert::{Alert, Severity};
use dioxus::prelude::*;

// Define severity priority for sorting
fn severity_priority(severity: &Severity) -> i32 {
    match severity.machinename.as_str() {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        _ => 3, // default for unknown or lower severities
    }
}

#[component]
pub fn AlertComponent(
    alert: Alert,
    is_pinned: bool,
    on_click: EventHandler<String>,
    on_mouse_enter: EventHandler<String>,
    on_mouse_leave: EventHandler<String>,
) -> Element {
    let id=alert.id.clone();
    // TODO: inset does not work
    let pinned_style = if is_pinned {"inset-shadow-xl/90"} else {""};
    let severity_class = format!("severity-{}", alert.severity.machinename);
    rsx! {
        tr {
            class: "alert whitespace-nowrap {severity_class} {pinned_style}",
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
            td { class: "p-1",
                a { href: alert.link, target: "_blank", "🔍" }
            }
            td { class: "p-1", "{alert.title}" }
            td { class: "p-1 truncate", "{alert.summary}" }
        
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
    sorted_alerts.sort_by_key(|alert| severity_priority(&alert.severity));

    rsx! {
        div { class: "overflow-x-scroll overflow-y-scroll pt-12",
            table { class: "border-separate border-spacing-0 table-auto min-w-full",
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
