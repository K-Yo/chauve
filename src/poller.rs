use chrono::{DateTime, Utc};

use crate::{
    entities::{alert::Alert, provider::Provider, settings::Settings},
    providers::grafana::provider::GrafanaProvider,
};

#[derive(Clone)]
pub struct Poller {
    last_poll_time: Option<DateTime<Utc>>,
    providers: Vec<Box<dyn Provider>>,
}

impl Poller {
    pub async fn poll(&mut self) -> Vec<Alert> {
        self.last_poll_time = Some(Utc::now());

        let provider = &self.providers.pop();
        match provider {
            Some(p) => p.alerts().await,
            _ => vec![]
        }
    }
    pub fn last_poll_time(&self) -> String {
        match self.last_poll_time {
            Some(date) => date.to_rfc3339(),
            _ => "Never".to_string(),
        }
    }
    pub fn alerts(&self) -> Vec<Alert> {
        vec![]
    }

    pub fn new(settings: Settings) -> Poller {
        let mut all_providers: Vec<Box<dyn Provider>> = vec![];
        for grafana_settings in settings.providers.grafana {
            let provider = GrafanaProvider::new(grafana_settings.url, grafana_settings.token);
            &all_providers.push(Box::new(provider));
        }
        Poller {
            last_poll_time: None,
            providers: all_providers,
        }
    }
}
