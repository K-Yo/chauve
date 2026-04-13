use config::Config;
use std::path::Path;

use crate::entities::settings::Settings;

/// Return app settings, creating a default Settings.toml if it does not exist.
pub fn get_settings() -> Settings {
    let settings_path = "Settings.toml";

    if !Path::new(settings_path).exists() {
        match toml::to_string(&Settings::default()) {
            Ok(toml_str) => {
                if let Err(e) = std::fs::write(settings_path, &toml_str) {
                    tracing::warn!("Failed to create default Settings.toml: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize default settings: {}", e);
            }
        }
    }

    let config = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("Settings"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    config.try_deserialize::<Settings>().unwrap()
}
