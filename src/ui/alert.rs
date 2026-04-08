use crate::entities::alert::{Alert, Severity};
use crate::poller::Poller;
use dioxus::prelude::*;

use dioxus_free_icons::icons::fa_brands_icons::FaGitlab;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCloud, FaCloudBolt, FaCloudRain, FaCloudShowersHeavy,
};
use dioxus_free_icons::Icon;

#[component]
fn GitlabIcon() -> Element {
    rsx!(
        Icon {
            width: None,
            height: None,
            icon: FaGitlab,
            style: "height: 1em",
        }
    )
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
            td {
                SeverityIcon { severity: alert.severity }
            }
            td { "{alert.title}" }
            td {
                a { href: alert.link, target: "_blank", GitlabIcon {} }
            }
        }
    }
}

#[component]
pub fn AlertList() -> Element {
    let alerts = consume_context::<Signal<Poller>>().read().alerts();
    rsx! {
        table {
            for alert in alerts {
                AlertComponent { alert }
            }
        }
    }
}
