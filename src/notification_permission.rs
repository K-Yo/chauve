//! Checks whether the OS is actually letting Chauve's notifications through.
//!
//! `mac-notification-sys` and `winrt-toast-reborn` can send toasts but don't
//! expose authorization/setting state, so each platform talks to the native
//! notification API directly for this one query. No equivalent exists on
//! Linux (D-Bus notification servers don't report a permission state).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotificationPermission {
    /// Only ever constructed on macOS/Windows, where a permission check exists.
    #[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
    Enabled,
    Disabled,
    /// No permission concept on this platform, or the check itself failed.
    Unsupported,
}

/// Blocking: run off the UI thread (e.g. via `spawn_blocking`).
pub fn check() -> NotificationPermission {
    #[cfg(target_os = "macos")]
    return check_macos();

    #[cfg(target_os = "windows")]
    return check_windows();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    NotificationPermission::Unsupported
}

#[cfg(target_os = "macos")]
fn check_macos() -> NotificationPermission {
    use block2::RcBlock;
    use objc2_user_notifications::{
        UNAuthorizationStatus, UNNotificationSettings, UNUserNotificationCenter,
    };
    use std::ptr::NonNull;
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();
    let center = unsafe { UNUserNotificationCenter::currentNotificationCenter() };
    let handler = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
        let status = unsafe { settings.as_ref().authorizationStatus() };
        let _ = tx.send(status);
    });
    unsafe { center.getNotificationSettingsWithCompletionHandler(&handler) };

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(UNAuthorizationStatus::Denied) => NotificationPermission::Disabled,
        Ok(status) => {
            tracing::debug!("macOS notification authorization status: {:?}", status);
            NotificationPermission::Enabled
        }
        Err(_) => {
            tracing::warn!("Timed out waiting for macOS notification settings");
            NotificationPermission::Unsupported
        }
    }
}

#[cfg(target_os = "windows")]
fn check_windows() -> NotificationPermission {
    use crate::settings::AUMID;
    use windows::UI::Notifications::{NotificationSetting, ToastNotificationManager};

    let setting = (|| -> windows::core::Result<NotificationSetting> {
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&AUMID.into())?;
        notifier.Setting()
    })();

    match setting {
        Ok(NotificationSetting::Enabled) => NotificationPermission::Enabled,
        Ok(other) => {
            tracing::debug!("Windows notification setting: {:?}", other);
            NotificationPermission::Disabled
        }
        Err(e) => {
            tracing::warn!("Failed to read Windows notification setting: {}", e);
            NotificationPermission::Unsupported
        }
    }
}
