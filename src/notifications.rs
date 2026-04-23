//! Desktop notification support.
//!
//! Sends OS-native notifications for new alerts.
//! Each platform uses its own richer API:
//!   - Linux:   notify-rust  (D-Bus / zbus, urgency levels)
//!   - macOS:   mac-notification-sys  (subtitle, per-severity sounds)
//!   - Windows: winrt-toast-reborn  (WinRT toast, sound, attribution text)

use crate::entities::alert::Alert;
#[cfg(target_os = "windows")]
use crate::settings::AUMID;
use crate::settings::toast_image_path;
use tracing::debug;

/// Fire-and-forget: sends a desktop notification for the given alert.
/// Failures are logged as warnings and do not affect the caller.
pub fn send_notification(alert: &Alert) {
    debug!("sending notification for alert {:?}", alert);
    let notif_alert = alert.clone();

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        send_linux(&notif_alert);

        #[cfg(target_os = "macos")]
        send_macos(&notif_alert);

        #[cfg(target_os = "windows")]
        send_windows(&notif_alert);
    });
}

// ---------------------------------------------------------------------------
// Linux — notify-rust via D-Bus
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn send_linux(alert: &Alert) {
    use notify_rust::Urgency;

    let urgency = match alert.severity.machinename.as_str() {
        "critical" | "high" => Urgency::Critical,
        "medium" => Urgency::Normal,
        _ => Urgency::Low,
    };

    let body = format!("{}\n{}", alert.summary, alert.description);
    let image_path = toast_image_path();

    if let Err(e) = notify_rust::Notification::new()
        .appname("Chauve")
        .summary(alert.title.as_str())
        .urgency(urgency)
        .body(&body)
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
fn send_macos(alert: &Alert) {
    use mac_notification_sys::Notification;

    let severity = alert.severity.machinename.as_str();
    let image_path = toast_image_path();
    let body = format!("{}\n{}", alert.summary, alert.description);

    let mut notification = Notification::new();
    notification
        .title(alert.title.as_str())
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
fn send_windows(alert: &Alert) {
    use winrt_toast_reborn::ToastDuration;
    use winrt_toast_reborn::content::audio::{LoopingSound, Sound};
    use winrt_toast_reborn::content::image::ImagePlacement;
    use winrt_toast_reborn::{Audio, Header, Image, Text, Toast, ToastManager};

    let manager = ToastManager::new(AUMID);

    let img = Image::new_local(toast_image_path())
        .unwrap()
        .with_placement(ImagePlacement::AppLogoOverride);
    // Map severity to a WinRT looping sound.
    let audio: Option<Audio> = match alert.severity.machinename.as_str() {
        "critical" => Some(Audio::new(Sound::Looping(LoopingSound::Alarm))),
        "high" => Some(Audio::new(Sound::Looping(LoopingSound::Alarm7))),
        _ => None,
    };

    let mut binding = Toast::new();
    let mut toast = binding
        .header(Header::new(
            "chauve_alerts",
            format!("{} alert", alert.severity.machinename),
            "",
        ))
        .text1(Text::new(alert.summary.as_str()))
        .text2(Text::new(alert.description.as_str()))
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
