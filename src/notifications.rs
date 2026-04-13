//! Desktop notification support.
//!
//! Sends OS-native notifications for new alerts using notify-rust.

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

    tokio::task::spawn_blocking(move || {
        if let Err(e) = notify_rust::Notification::new()
            .appname("chauve")
            .summary(&summary)
            .body(&body)
            .show()
        {
            tracing::warn!("Failed to send desktop notification: {}", e);
        }
    });
}
