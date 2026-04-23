use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Settings {
    #[serde(default)]
    pub providers: ProvidersSettings,

    #[serde(default = "default_poll_frequency")]
    pub poll_frequency: u64,

    #[serde(default)]
    pub notifications: NotificationSettings,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotificationSettings {
    #[serde(default = "default_notification_enabled")]
    pub enabled: bool,
    /// Minimum severity to notify. Alerts at this severity or more severe will trigger a notification.
    /// Recognized values (most to least severe): critical, high, medium, low.
    #[serde(default = "default_notification_min_severity")]
    pub min_severity: String,
}

fn default_notification_enabled() -> bool {
    true
}

fn default_notification_min_severity() -> String {
    "high".to_string()
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: default_notification_enabled(),
            min_severity: default_notification_min_severity(),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct ProvidersSettings {
    #[serde(default)]
    pub grafana: Vec<ProviderGrafanaSetting>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProviderGrafanaSetting {
    pub url: String,
    pub token: String,
}

// Default poll frequency in seconds (5 seconds)
fn default_poll_frequency() -> u64 {
    5
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            providers: ProvidersSettings::default(),
            poll_frequency: default_poll_frequency(),
            notifications: NotificationSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use config::{Config, File, FileFormat};

    use super::*;

    #[test]
    fn valid_config_parse() {
        let config = Config::builder()
            .add_source(File::from_str(
                "poll_frequency=10\n\
                     [[providers.grafana]]\n\
                        url=\"http://localhost:3000\"\n\
                        token=\"glsa_Som3t0k3n\"\n",
                FileFormat::Toml,
            ))
            .build()
            .unwrap();

        let settings = config.try_deserialize::<Settings>().unwrap();

        assert_eq!(
            settings.providers.grafana[0].url.as_str(),
            "http://localhost:3000"
        );
        assert_eq!(
            settings.providers.grafana[0].token.as_str(),
            "glsa_Som3t0k3n"
        );
        assert_eq!(settings.poll_frequency, 10);
    }

    #[test]
    fn empty_config_parse() {
        let config = Config::builder()
            .add_source(File::from_str("\n", FileFormat::Toml))
            .build()
            .unwrap();

        let settings = config.try_deserialize::<Settings>().unwrap();
        assert_eq!(settings.poll_frequency, 5); // Default value
    }
}
