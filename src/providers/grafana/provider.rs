use super::alert::GrafanaAlert;
use crate::entities::{alert::Alert, provider::{Provider, ProviderError, convert_alert}};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct GrafanaProvider {
    _url: String,   // http://172.31.31.240:3000
    _token: String, // glsa-...
}

impl GrafanaProvider {
    pub fn new(url: String, token: String) -> GrafanaProvider {
        GrafanaProvider {
            _url: url,
            _token: token,
        }
    }
}

#[async_trait]
impl Provider for GrafanaProvider {
    async fn alerts(&self) -> Result<Vec<Alert>, ProviderError> {
        let generic_alerts = self.pull().await?.iter().map(convert_alert).collect::<Vec<Alert>>();
        Ok(generic_alerts)
    }
    fn clone_box(&self) -> Box<dyn Provider> {
        Box::new(self.clone())
    }
}

impl GrafanaProvider {
    async fn pull(&self) -> Result<Vec<GrafanaAlert>, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/alertmanager/grafana/api/v2/alerts", self._url);
        let response = client
            .get(url)
            .bearer_auth(self._token.clone())
            .send()
            .await?
            .json::<Vec<GrafanaAlert>>()
            .await?;

        
        Ok(response)
    }
}
