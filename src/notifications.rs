//! Desktop notification support.
//!
//! Sends OS-native notifications for new alerts, at most one per poll:
//! a single new alert is shown with its details, several are batched into
//! a summary counting alerts per severity.
//! Each platform uses its own richer API:
//!   - Linux:   notify-rust  (D-Bus / zbus, urgency levels)
//!   - macOS:   mac-notification-sys  (subtitle, per-severity sounds)
//!   - Windows: winrt-toast-reborn  (WinRT toast, sound, attribution text)

use std::collections::BTreeMap;

use crate::entities::alert::Alert;
#[cfg(target_os = "windows")]
use crate::settings::AUMID;
use crate::settings::toast_image_path;
use tracing::debug;

/// Platform-agnostic content of one desktop notification.
#[derive(Debug, Clone, PartialEq)]
struct Notice {
    title: String,
    /// Header line (Windows toast header).
    header: String,
    /// Severity machinename driving urgency and sound: the most severe one
    /// for a batch.
    severity: String,
    summary: String,
    description: String,
}

impl Notice {
    fn body(&self) -> String {
        if self.description.is_empty() {
            self.summary.clone()
        } else {
            format!("{}\n{}", self.summary, self.description)
        }
    }
}

/// Build the single notification for this batch of new alerts, if any.
fn build_notice(alerts: &[&Alert]) -> Option<Notice> {
    match alerts {
        [] => None,
        [alert] => Some(Notice {
            title: alert.title.clone(),
            header: format!("{} alert", alert.severity.machinename),
            severity: alert.severity.machinename.clone(),
            summary: alert.summary.clone(),
            description: alert.description.clone(),
        }),
        _ => {
            // Most severe first, then by name for a stable order.
            let mut counts: BTreeMap<(u8, &str), usize> = BTreeMap::new();
            for alert in alerts {
                let key = (alert.severity.order(), alert.severity.machinename.as_str());
                *counts.entry(key).or_default() += 1;
            }
            let severity = counts
                .keys()
                .next()
                .map(|(_, name)| name.to_string())
                .unwrap_or_default();
            let summary = counts
                .iter()
                .map(|((_, name), n)| format!("{n} {name}"))
                .collect::<Vec<_>>()
                .join(", ");
            let title = format!("{} new alerts", alerts.len());
            Some(Notice {
                header: title.clone(),
                title,
                severity,
                summary,
                description: String::new(),
            })
        }
    }
}

/// Fire-and-forget: sends at most one desktop notification for the given
/// new alerts. Failures are logged as warnings and do not affect the caller.
pub fn send_notifications(alerts: &[&Alert]) {
    let Some(notice) = build_notice(alerts) else {
        return;
    };
    debug!("sending notification {:?}", notice);

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        send_linux(&notice);

        #[cfg(target_os = "macos")]
        send_macos(&notice);

        #[cfg(target_os = "windows")]
        send_windows(&notice);
    });
}

// ---------------------------------------------------------------------------
// Linux — notify-rust via D-Bus
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn send_linux(notice: &Notice) {
    use notify_rust::Urgency;

    let urgency = match notice.severity.as_str() {
        "critical" | "high" => Urgency::Critical,
        "medium" => Urgency::Normal,
        _ => Urgency::Low,
    };

    let image_path = toast_image_path();

    if let Err(e) = notify_rust::Notification::new()
        .appname("Chauve")
        .summary(notice.title.as_str())
        .urgency(urgency)
        .body(&notice.body())
        .image_path(image_path.to_str().unwrap_or(""))
        .show()
    {
        tracing::warn!("Failed to send desktop notification: {}", e);
    }
}

// ---------------------------------------------------------------------------
// macOS — mac-notification-sys
// Supports subtitle, per-severity sounds, and direct Notification Center API.
// Sounds from the macOS Monterey+ set.
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn send_macos(notice: &Notice) {
    use mac_notification_sys::Notification;

    let severity = notice.severity.as_str();
    let image_path = toast_image_path();
    let body = notice.body();

    let mut notification = Notification::new();
    notification
        .title(notice.title.as_str())
        .subtitle(severity)
        .message(&body)
        .app_icon(image_path.to_str().unwrap_or(""));

    // macOS Monterey+ system sounds, chosen by urgency feel.
    match severity {
        "critical" => {
            notification.sound("Crystal");
        }
        "high" => {
            notification.sound("Spell");
        }
        "medium" => {
            notification.sound("Bubble");
        }
        _ => {}
    }

    if let Err(e) = notification.send() {
        tracing::warn!("Failed to send desktop notification: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Windows — winrt-toast-reborn
// Supports sound selection, duration, and attribution text.
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn send_windows(notice: &Notice) {
    use winrt_toast_reborn::ToastDuration;
    use winrt_toast_reborn::content::audio::{LoopingSound, Sound};
    use winrt_toast_reborn::content::image::ImagePlacement;
    use winrt_toast_reborn::{Audio, Header, Image, Text, Toast, ToastManager};

    let manager = ToastManager::new(AUMID);

    let img = Image::new_local(toast_image_path())
        .unwrap()
        .with_placement(ImagePlacement::AppLogoOverride);
    // Map severity to a WinRT looping sound.
    let audio: Option<Audio> = match notice.severity.as_str() {
        "critical" => Some(Audio::new(Sound::Looping(LoopingSound::Alarm))),
        "high" => Some(Audio::new(Sound::Looping(LoopingSound::Alarm7))),
        _ => None,
    };

    let mut binding = Toast::new();
    let mut toast = binding
        .header(Header::new("chauve_alerts", notice.header.as_str(), ""))
        .text1(Text::new(notice.summary.as_str()))
        .text2(Text::new(notice.description.as_str()))
        .image(1, img)
        .duration(ToastDuration::Long)
        .launch("");

    if let Some(a) = audio {
        toast = toast.audio(a);
    }

    if let Err(e) = manager.show(&toast) {
        tracing::warn!("Failed to send desktop notification: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::alert::Severity;

    fn alert(id: &str, severity: &str) -> Alert {
        Alert {
            id: id.to_string(),
            title: format!("Alert {id}"),
            severity: Severity::new(severity.to_string()),
            description: format!("description {id}"),
            summary: format!("summary {id}"),
            instance: String::new(),
            starts_at: None,
            variants: vec![],
        }
    }

    #[test]
    fn no_alert_no_notice() {
        assert_eq!(build_notice(&[]), None);
    }

    #[test]
    fn single_alert_keeps_details() {
        let a = alert("1", "high");
        let notice = build_notice(&[&a]).unwrap();
        assert_eq!(notice.title, "Alert 1");
        assert_eq!(notice.header, "high alert");
        assert_eq!(notice.severity, "high");
        assert_eq!(notice.body(), "summary 1\ndescription 1");
    }

    #[test]
    fn several_alerts_are_counted_by_severity() {
        let alerts = [
            alert("1", "medium"),
            alert("2", "critical"),
            alert("3", "medium"),
            alert("4", "low"),
            alert("5", "critical"),
            alert("6", "medium"),
        ];
        let refs: Vec<&Alert> = alerts.iter().collect();
        let notice = build_notice(&refs).unwrap();
        assert_eq!(notice.title, "6 new alerts");
        assert_eq!(notice.header, "6 new alerts");
        assert_eq!(notice.severity, "critical");
        assert_eq!(notice.body(), "2 critical, 3 medium, 1 low");
    }
}
