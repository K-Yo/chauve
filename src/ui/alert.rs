use crate::entities::alert::{Alert, Severity};
use crate::poller::Poller;
use dioxus::prelude::*;

use dioxus_free_icons::icons::fa_solid_icons::{
    FaCloud, FaCloudBolt, FaCloudRain, FaCloudShowersHeavy, FaMagnifyingGlass
};
use dioxus_free_icons::Icon;

#[component]
fn GitlabIcon() -> Element {
    rsx!(
        Icon {
            width: None,
            height: None,
            icon: FaMagnifyingGlass,
            style: "height: 1em",
        }
    )
}

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
fn SeverityIcon(severity: Severity) -> Element {
    let str_severity = severity.machinename.as_str();
    match str_severity {
        "critical" => rsx!(
            Icon {
                width: None,
                height: None,
                icon: FaCloudBolt,
                style: "height: 1em",
            }
        ),
        "high" => rsx!(
            Icon {
                width: None,
                height: None,
                icon: FaCloudShowersHeavy,
                style: "height: 1em",
            }
        ),
        "medium" => rsx!(
            Icon {
                width: None,
                height: None,
                icon: FaCloudRain,
                style: "height: 1em",
            }
        ),
        _ => rsx!(
            Icon {
                width: None,
                height: None,
                icon: FaCloud,
                style: "height: 1em",
            }
        ),
    }
}

#[component]
pub fn AlertComponent(alert: Alert) -> Element {
    let severity_class = format!("severity-{}", alert.severity.machinename);
    rsx! {
        tr { class: "alert {severity_class}",
            td { class: "p-1",
                SeverityIcon { severity: alert.severity }
            }
            td { class: "p-1", "{alert.title}" }
            td { class: "p-1", "{alert.summary}" }
            td { class: "p-1",
                a { href: alert.link, target: "_blank", GitlabIcon {} }
            }
        }
    }
}

#[component]
pub fn AlertList() -> Element {
    let alerts = consume_context::<Signal<Poller>>().read().alerts();

    // Sort alerts by severity, with critical first
    let mut sorted_alerts = alerts.clone();
    sorted_alerts.sort_by_key(|alert| severity_priority(&alert.severity));

    rsx! {
        div { class: "overflow-x-auto",
            table { class: "table-auto w-full",
                for alert in sorted_alerts {
                    AlertComponent { alert }
                }
            }
        }

    }
}
