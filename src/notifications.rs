//! Desktop notification support.
//!
//! Sends OS-native notifications for new alerts.
//! Each platform uses its own richer API:
//!   - Linux:   notify-rust  (D-Bus / zbus, urgency levels)
//!   - macOS:   mac-notification-sys  (subtitle, per-severity sounds)
//!   - Windows: winrt-toast-reborn  (WinRT toast, sound, attribution text)

use crate::entities::alert::Alert;

/// Fire-and-forget: sends a desktop notification for the given alert.
/// Failures are logged as warnings and do not affect the caller.
pub fn send_notification(alert: &Alert) {
    let summary = format!("[{}] {}", alert.severity.machinename, alert.title);
    let body = if !alert.summary.is_empty() {
        alert.summary.clone()
    } else {
        alert.description.clone()
    };
    let severity = alert.severity.machinename.clone();

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        send_linux(&summary, &body, &severity);

        #[cfg(target_os = "macos")]
        send_macos(&summary, &body, &severity);

        #[cfg(target_os = "windows")]
        send_windows(&summary, &body, &severity);
    });
}

// ---------------------------------------------------------------------------
// Linux — notify-rust via D-Bus
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn send_linux(summary: &str, body: &str, severity: &str) {
    use notify_rust::Urgency;

    let urgency = match severity {
        "critical" | "high" => Urgency::Critical,
        "medium" => Urgency::Normal,
        _ => Urgency::Low,
    };

    if let Err(e) = notify_rust::Notification::new()
        .appname("chauve")
        .summary(summary)
        .urgency(urgency)
        .body(body)
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
fn send_macos(summary: &str, body: &str, severity: &str) {
    use mac_notification_sys::{NotificationOptions, send_notification as macos_notify};

    // macOS Monterey+ system sounds, chosen by urgency feel.
    let sound = match severity {
        "critical" => "Crystal",   // sharp, attention-grabbing
        "high" => "Spell",         // prominent but not jarring
        "medium" => "Bubble",      // gentle mid-tier alert
        _ => "",                   // no sound for low/unknown
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
fn send_windows(summary: &str, body: &str, severity: &str) {
    use winrt_toast_reborn::{Audio, Header, Text, Toast, ToastManager};
    use winrt_toast_reborn::content::audio::{Sound,LoopingSound};

    let manager = ToastManager::new("Chauve.Alerts");

    // Map severity to a WinRT looping sound.
    let audio: Option<Audio> = match severity {
        "critical" | "high" => Some(Audio::new(Sound::Looping(LoopingSound::Alarm))),
        "medium" => Some(Audio::new(Sound::Looping(LoopingSound::Call))),
        _ => None,
    };

    let mut toast = Toast::new()
        .header(Header::new("chauve", "Chauve", ""))
        .text1(Text::new(summary))
        .text2(Text::new(body))
        .text3(Text::new(severity).info_attribution());

    if let Some(a) = audio {
        toast = toast.audio(a);
    }

    if let Err(e) = manager.show(&toast) {
        tracing::warn!("Failed to send desktop notification: {}", e);
    }
}
