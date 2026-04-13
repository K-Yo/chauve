use config::Config;
use directories::ProjectDirs;
use std::path::PathBuf;

use crate::entities::settings::Settings;

/// Returns the OS-appropriate path for the settings file.
///
/// - Linux:   ~/.config/chauve/Settings.toml
/// - macOS:   ~/Library/Application Support/chauve/Settings.toml
/// - Windows: %APPDATA%\chauve\Settings.toml
///
/// Falls back to ./Settings.toml if the config dir cannot be determined.
pub fn settings_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "chauve") {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            tracing::warn!("Failed to create config directory: {}", e);
            return PathBuf::from("Settings.toml");
        }
        config_dir.join("Settings.toml")
    } else {
        PathBuf::from("Settings.toml")
    }
}

/// Return app settings, creating a default settings file if it does not exist.
pub fn get_settings() -> Settings {
    let path = settings_path();

    if !path.exists() {
        match toml::to_string(&Settings::default()) {
            Ok(toml_str) => {
                if let Err(e) = std::fs::write(&path, &toml_str) {
                    tracing::warn!("Failed to create default settings at {}: {}", path.display(), e);
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
