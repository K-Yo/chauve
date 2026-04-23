use config::Config;
use directories::ProjectDirs;
use std::path::PathBuf;

use crate::entities::settings::Settings;

pub const AUMID: &str = "com.github.k-yo.chauve";
/// Path on the image for notifications
pub fn toast_image_path() -> PathBuf {
    config_dir().join("fire_extinguisher.png")
}
/// Returns the OS-appropriate path for the config directory.
///
/// - Linux:   ~/.config/chauve/
/// - macOS:   ~/Library/Application Support/chauve/
/// - Windows: %APPDATA%\chauve\
///
/// Falls back to . if the config dir cannot be determined.
pub fn config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "chauve") {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            tracing::warn!("Failed to create config directory: {}", e);
            return PathBuf::from(".");
        }
        config_dir
    } else {
        PathBuf::from(".")
    }
}

pub fn settings_path() -> PathBuf {
    config_dir().join("Settings.toml")
}

/// Return app settings, creating a default settings file if it does not exist.
pub fn get_settings() -> Settings {
    let path = settings_path();

    if !path.exists() {
        match toml::to_string(&Settings::default()) {
            Ok(toml_str) => {
                if let Err(e) = std::fs::write(&path, &toml_str) {
                    tracing::warn!(
                        "Failed to create default settings at {}: {}",
                        path.display(),
                        e
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize default settings: {}", e);
            }
        }
    }

    let config = Config::builder()
        .add_source(config::File::from(path.as_path()))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    config.try_deserialize::<Settings>().unwrap()
}
