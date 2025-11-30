use super::alert::GrafanaAlert;
use crate::provider::Provider;
use async_trait::async_trait;

#[derive(Debug)]
pub struct GrafanaProvider {
    url: String,   // http://172.31.31.240:3000
    token: String, // glsa-...
}

#[async_trait]
impl Provider<GrafanaAlert> for GrafanaProvider {
    async fn pull(&self) -> Vec<GrafanaAlert> {
        let client = reqwest::Client::new();
        let url = "{self.url}/api/alertmanager/grafana/api/v2/alerts";
        let response = client
            .get(url)
            .bearer_auth(self.token.clone())
            .send()
            .await
            .unwrap();
        response.json::<Vec<GrafanaAlert>>().await.unwrap()
    }
}
