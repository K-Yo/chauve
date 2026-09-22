#[cfg(target_os = "windows")]
use crate::settings::AUMID;
use crate::settings::config_dir;
use std::{fs::File, io::Write};

pub fn register_app() {
    setup_images();

    #[cfg(target_os = "linux")]
    setup_linux();

    #[cfg(target_os = "windows")]
    setup_windows();

    #[cfg(target_os = "macos")]
    setup_macos();
}

pub fn setup_images() {
    let notification_image = include_bytes!("../assets/img/fire-extinguisher_1f9ef.png");
    let config_dir = config_dir();
    let mut alert_image = File::create(config_dir.join("fire_extinguisher.png")).unwrap();
    let _ = alert_image.write_all(notification_image);
}

#[cfg(target_os = "windows")]
pub fn setup_windows() {
    // register the app ID for toast notifications
    let _ = winrt_toast_reborn::register(AUMID, "Chauve", None);
}

#[cfg(target_os = "linux")]
pub fn setup_linux() {}

#[cfg(target_os = "macos")]
pub fn setup_macos() {}

pub fn icon_bytes() -> &'static [u8] {
    include_bytes!("../assets/icons/icon.png")
}
