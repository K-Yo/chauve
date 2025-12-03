use config::Config;

use crate::entities::settings::Settings;

/// Return app settings
pub fn get_settings() -> Settings {
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
