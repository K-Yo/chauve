//! Desktop notification support.
//!
//! Sends OS-native notifications for new alerts.
//! Each platform uses its own richer API:
//!   - Linux:   notify-rust  (D-Bus / zbus, urgency levels)
//!   - macOS:   mac-notification-sys  (subtitle, per-severity sounds)
//!   - Windows: winrt-toast-reborn  (WinRT toast, sound, attribution text)

use crate::entities::alert::Alert;
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
        send_linux(notif_alert);

        #[cfg(target_os = "macos")]
        send_macos(notif_alert);

        #[cfg(target_os = "windows")]
        send_windows(&notif_alert);
    });
}

// ---------------------------------------------------------------------------
// Linux — notify-rust via D-Bus
// ---------------------------------------------------------------------------

// #[cfg(target_os = "linux")]
fn send_linux(alert: &Alert) {
    use notify_rust::Urgency;

    let urgency = match alert.severity.machinename.as_str() {
        "critical" | "high" => Urgency::Critical,
        "medium" => Urgency::Normal,
        _ => Urgency::Low,
    };

    if let Err(e) = notify_rust::Notification::new()
        .appname("Chauve")
        .summary(alert.title.as_str())
        .urgency(urgency)
        .body(alert.summary.as_str())
        // TODO: test image
        .image_path(toast_image_path().as_path().to_str().unwrap())
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
fn send_macos(salert: &Alert) {
    use mac_notification_sys::{NotificationOptions, send_notification as macos_notify};

    // macOS Monterey+ system sounds, chosen by urgency feel.
    let sound = match severity {
        "critical" => "Crystal", // sharp, attention-grabbing
        "high" => "Spell",       // prominent but not jarring
        "medium" => "Bubble",    // gentle mid-tier alert
        _ => "",                 // no sound for low/unknown
    };

    let options = NotificationOptions {
        subtitle: Some(severity),
        sound: Some(sound),
        ..Default::default()
    };

    if let Err(e) = macos_notify("chauve", None, summary, Some(body), &options) {
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
        .header(Header::new("chauve_alerts", format!("{} alert", alert.severity.machinename), ""))
        .text1(Text::new(alert.summary.as_str()))
        .image(1, img)
        .duration(ToastDuration::Long)
        // TODO how to open the default browser on click?
        // .action(Action::new("See on app", alert.id.clone(), "view_alert").with_activation_type(ActivationType::Protocol))
        ;

    if let Some(a) = audio {
        toast = toast.audio(a);
    }

    if let Err(e) = manager.show(&toast) {
        tracing::warn!("Failed to send desktop notification: {}", e);
    }
}
