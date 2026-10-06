use crate::entities::alert::{Alert, AlertVariant, format_age};
use crate::poller::Poller;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use std::cmp::Reverse;

/// Provider link columns, in display order (matched against `AlertVariant::label`).
const LINK_COLUMNS: [&str; 2] = ["Grafana", "GitLab"];

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
    // One slot per provider column, (href, tooltip, icon) when that provider links this alert.
    let links: Vec<Option<(String, String, &str)>> = LINK_COLUMNS
        .iter()
        .map(|label| {
            alert
                .variants
                .iter()
                .find(|v| v.label() == *label)
                .and_then(|v| {
                    v.link()
                        .map(|href| (href.to_string(), format!("Open in {}", v.label()), v.icon()))
                })
        })
        .collect();
    let assignee = alert.variants.iter().find_map(|v| match v {
        AlertVariant::Gitlab(v) => v.assignee.clone(),
        _ => None,
    });
    // GitLab alert offered for self-assignment when nobody holds it.
    let unassigned = alert
        .variants
        .iter()
        .find(|v| matches!(v, AlertVariant::Gitlab(_)))
        .cloned();
    let mut poller_signal: Signal<Poller> = use_context();
    let mut assigning = use_signal(|| false);
    let mut assign_error: Signal<Option<String>> = use_signal(|| None);
    let assign_title = match assign_error() {
        Some(error) => format!("Assignment failed: {error}"),
        None => "Assign to me".to_string(),
    };
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
            // One fixed-width column per provider, so icons stay aligned across rows.
            div { class: "p-1 flex-none flex gap-1",
                for link in links {
                    div { class: "w-6 flex-none text-center",
                        if let Some((href, title, icon)) = link {
                            a { href, target: "_blank", title, "{icon}" }
                        }
                    }
                }
            }
            // GitLab assignee avatar, sized to the text line so the row keeps its height.
            div { class: "p-1 flex-none w-8",
                if let Some(user) = assignee {
                    if let Some(src) = user.avatar_url {
                        img {
                            class: "block size-6 rounded-full object-cover",
                            src,
                            alt: "{user.name}",
                            title: "Assigned to {user.name} (@{user.username})",
                        }
                    } else {
                        div {
                            class: "size-6 text-center",
                            title: "Assigned to {user.name} (@{user.username})",
                            "👤"
                        }
                    }
                } else if let Some(variant) = unassigned {
                    button {
                        class: "block size-6 text-center cursor-pointer disabled:cursor-wait",
                        disabled: assigning(),
                        title: "{assign_title}",
                        onclick: move |event| {
                            // The row pins on click: assigning must not toggle it.
                            event.stop_propagation();
                            let variant = variant.clone();
                            spawn(async move {
                                assigning.set(true);
                                let poller = poller_signal.read().clone();
                                match poller.assign_to_me(&variant).await {
                                    Ok(updated) => {
                                        assign_error.set(None);
                                        poller_signal.write().update_variant(&variant, updated);
                                    }
                                    Err(error) => {
                                        tracing::warn!("Assignment failed: {}", error);
                                        assign_error.set(Some(error.to_string()));
                                    }
                                }
                                assigning.set(false);
                            });
                        },
                        if assigning() {
                            "⏳"
                        } else if assign_error().is_some() {
                            "⚠️"
                        } else {
                            "🙋"
                        }
                    }
                }
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
    // Sort alerts by severity (critical first), then most recently started first.
    // Alerts without a start time land last within their severity group.
    let mut sorted_alerts = alerts.clone();
    sorted_alerts.sort_by_key(|alert| (alert.severity.order(), Reverse(alert.starts_at)));

    // One reference instant for the whole list so every row agrees on "now".
    let now = Utc::now();

    rsx! {
        div { class: "overflow-x-scroll overflow-y-scroll pt-12 pb-6",
            div { class: "min-w-full",
                for alert in sorted_alerts {
                    AlertComponent {
                        // Rows reorder between polls: keep per-row state with its alert.
                        key: "{alert.id}",
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
