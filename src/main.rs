//! Chauve, a notification system.
mod entities;
mod use_cases;
// use crate::entities::alert_providers::AlertProvider;
// use crate::use_cases::pull_alerts::PullAlerts;
use crate::entities::settings::Settings;

use config::Config;

fn main() {
    let config = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("Settings"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    let settings = config
            .try_deserialize::<Settings>()
            .unwrap();
    // Print out our settings (as a HashMap)
    println!(
        "{:?}",
        settings
    );

    // settings.providers
    // let alert_source = AlertProvider {
    //     name: String::from("grafana"),
    // };
    // dbg!(&alert_source);
    // let alert_list = alert_source.pull();
    // dbg!(alert_list);
}
