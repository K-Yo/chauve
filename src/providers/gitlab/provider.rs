use super::alert::GitlabAlert;
use crate::entities::{
    alert::Alert,
    provider::{Provider, ProviderError, convert_alert},
};
use anyhow::anyhow;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

/// Open alerts of a project. GitLab has no REST endpoint listing alerts.
///
/// The sort is required: without it, filtering on statuses times out on
/// projects with a large history of resolved alerts.
const ALERTS_QUERY: &str = "query($ids: [ID!]) { projects(ids: $ids) { nodes {
  alertManagementAlerts(statuses: [TRIGGERED, ACKNOWLEDGED], sort: CREATED_DESC, first: 100) {
    nodes { iid title severity description startedAt webUrl details }
  } } } }";

#[derive(Debug, Clone)]
pub struct GitlabProvider {
    url: String,        // https://gitlab.com
    token: String,      // glpat-...
    project_id: String, // 12345
}

impl GitlabProvider {
    pub fn new(url: String, token: String, project_id: String) -> GitlabProvider {
        GitlabProvider {
            url: url.trim_end_matches('/').to_string(),
            token,
            project_id,
        }
    }
}

#[async_trait]
impl Provider for GitlabProvider {
    async fn alerts(&self) -> Result<Vec<Alert>, ProviderError> {
        let generic_alerts = self
            .pull()
            .await?
            .iter()
            .map(convert_alert)
            .collect::<Vec<Alert>>();
        Ok(generic_alerts)
    }
    fn clone_box(&self) -> Box<dyn Provider> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Deserialize)]
struct GraphqlResponse {
    data: Option<GraphqlData>,
    #[serde(default)]
    errors: Vec<GraphqlError>,
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GraphqlData {
    projects: Nodes<Project>,
}

#[derive(Debug, Deserialize)]
struct Project {
    #[serde(rename = "alertManagementAlerts")]
    alert_management_alerts: Nodes<GitlabAlert>,
}

#[derive(Debug, Deserialize)]
struct Nodes<T> {
    nodes: Vec<T>,
}

impl GitlabProvider {
    async fn pull(&self) -> Result<Vec<GitlabAlert>, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/graphql", self.url);
        let body = json!({
            "query": ALERTS_QUERY,
            "variables": { "ids": [format!("gid://gitlab/Project/{}", self.project_id)] },
        });
        let response = client
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<GraphqlResponse>()
            .await?;

        if !response.errors.is_empty() {
            let messages: Vec<String> = response.errors.into_iter().map(|e| e.message).collect();
            return Err(anyhow!("GitLab GraphQL error: {}", messages.join("; ")).into());
        }

        let project = response
            .data
            .and_then(|data| data.projects.nodes.into_iter().next())
            .ok_or_else(|| {
                ProviderError::Config(format!(
                    "GitLab project {} not found or not accessible",
                    self.project_id
                ))
            })?;

        let alerts = project
            .alert_management_alerts
            .nodes
            .into_iter()
            .map(|mut alert| {
                alert.project_id = self.project_id.clone();
                alert
            })
            .collect();
        Ok(alerts)
    }
}
