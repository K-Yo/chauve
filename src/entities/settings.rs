use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Settings {
    #[serde(default)]
    pub providers: ProvidersSettings,
    
    #[serde(default = "default_poll_frequency")]
    pub poll_frequency: u64,
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

#[cfg(test)]
mod tests {
    use config::{Config, File, FileFormat};

    use super::*;

    #[test]
    fn valid_config_parse() {
        let config = Config::builder()
            .add_source(File::from_str(
                
                    "[[providers.grafana]]\n\
                        url=\"http://localhost:3000\"\n\
                        token=\"glsa_Som3t0k3n\"\n\
                        poll_frequency=10\n"
                ,
                FileFormat::Toml,
            ))
            .build()
            .unwrap();

    let settings = config.try_deserialize::<Settings>().unwrap();

        assert_eq!(settings.providers.grafana[0].url.as_str(), "http://localhost:3000");
        assert_eq!(settings.providers.grafana[0].token.as_str(), "glsa_Som3t0k3n");
        assert_eq!(settings.poll_frequency, 10);
    }

    #[test]
    fn empty_config_parse() {
        let config = Config::builder()
            .add_source(File::from_str(
                
                    "\n"
                ,
                FileFormat::Toml,
            ))
            .build()
            .unwrap();

        let settings = config.try_deserialize::<Settings>().unwrap();
        assert_eq!(settings.poll_frequency, 5); // Default value
    }
}
