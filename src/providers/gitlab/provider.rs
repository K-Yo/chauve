use super::alert::{GitlabAlert, GitlabAssignee};
use crate::entities::{
    alert::{Alert, AlertVariant, GitlabUser},
    provider::{Provider, ProviderError, convert_alert},
};
use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

/// Open alerts of a project. GitLab has no REST endpoint listing alerts.
///
/// The sort is required: without it, filtering on statuses times out on
/// projects with a large history of resolved alerts.
const ALERTS_QUERY: &str = "query($ids: [ID!]) { projects(ids: $ids) { nodes { fullPath
  alertManagementAlerts(statuses: [TRIGGERED, ACKNOWLEDGED], sort: CREATED_DESC, first: 100) {
    nodes { iid title severity description startedAt webUrl details
      assignees { nodes { name username avatarUrl } } }
  } } } }";

const CURRENT_USER_QUERY: &str = "{ currentUser { username } }";

/// APPEND rather than REPLACE: never unassign someone who took the alert meanwhile.
const ASSIGN_MUTATION: &str = "mutation($projectPath: ID!, $iid: String!, $usernames: [String!]!) {
  alertSetAssignees(input: { projectPath: $projectPath, iid: $iid,
    assigneeUsernames: $usernames, operationMode: APPEND }) {
    errors alert { assignees { nodes { name username avatarUrl } } }
  } }";

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

    async fn assign_to_me(
        &self,
        variant: &AlertVariant,
    ) -> Result<Option<AlertVariant>, ProviderError> {
        let AlertVariant::Gitlab(gitlab_variant) = variant else {
            return Ok(None);
        };
        if gitlab_variant.project_id != self.project_id
            || !gitlab_variant.web_url.starts_with(&self.url)
        {
            return Ok(None);
        }

        let username = self.current_username().await?;
        let payload = self
            .graphql::<AssignData>(
                ASSIGN_MUTATION,
                json!({
                    "projectPath": gitlab_variant.project_path,
                    "iid": gitlab_variant.iid,
                    "usernames": [username],
                }),
            )
            .await?
            .alert_set_assignees
            .ok_or_else(|| anyhow!("GitLab returned no result for the assignment"))?;
        if !payload.errors.is_empty() {
            return Err(anyhow!(
                "GitLab refused the assignment: {}",
                payload.errors.join("; ")
            )
            .into());
        }

        let mut updated = gitlab_variant.clone();
        updated.assignee = payload.alert.and_then(|mut alert| {
            alert
                .assignees
                .nodes
                .iter_mut()
                .for_each(|user| self.absolute_avatar(user));
            alert.assignees.nodes.first().map(GitlabUser::from)
        });
        Ok(Some(AlertVariant::Gitlab(updated)))
    }

    fn clone_box(&self) -> Box<dyn Provider> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Deserialize)]
struct GraphqlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphqlError>,
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct AlertsData {
    projects: Nodes<Project>,
}

#[derive(Debug, Deserialize)]
struct Project {
    #[serde(rename = "fullPath")]
    full_path: String,
    #[serde(rename = "alertManagementAlerts")]
    alert_management_alerts: Nodes<GitlabAlert>,
}

#[derive(Debug, Deserialize)]
struct CurrentUserData {
    #[serde(rename = "currentUser")]
    current_user: Option<CurrentUser>,
}

#[derive(Debug, Deserialize)]
struct CurrentUser {
    username: String,
}

#[derive(Debug, Deserialize)]
struct AssignData {
    #[serde(rename = "alertSetAssignees")]
    alert_set_assignees: Option<AssignPayload>,
}

#[derive(Debug, Deserialize)]
struct AssignPayload {
    #[serde(default)]
    errors: Vec<String>,
    alert: Option<AssignedAlert>,
}

#[derive(Debug, Deserialize)]
struct AssignedAlert {
    #[serde(default)]
    assignees: Nodes<GitlabAssignee>,
}

#[derive(Debug, Deserialize)]
pub struct Nodes<T> {
    pub nodes: Vec<T>,
}

impl<T> Default for Nodes<T> {
    fn default() -> Self {
        Nodes { nodes: Vec::new() }
    }
}

impl GitlabProvider {
    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Value,
    ) -> Result<T, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/graphql", self.url);
        let body = json!({ "query": query, "variables": variables });
        let response = client
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<GraphqlResponse<T>>()
            .await?;

        if !response.errors.is_empty() {
            let messages: Vec<String> = response.errors.into_iter().map(|e| e.message).collect();
            return Err(anyhow!("GitLab GraphQL error: {}", messages.join("; ")).into());
        }
        response
            .data
            .ok_or_else(|| anyhow!("GitLab GraphQL response has no data").into())
    }

    async fn pull(&self) -> Result<Vec<GitlabAlert>, ProviderError> {
        let variables = json!({ "ids": [format!("gid://gitlab/Project/{}", self.project_id)] });
        let project = self
            .graphql::<AlertsData>(ALERTS_QUERY, variables)
            .await?
            .projects
            .nodes
            .into_iter()
            .next()
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
                alert.project_path = project.full_path.clone();
                alert
                    .assignees
                    .nodes
                    .iter_mut()
                    .for_each(|user| self.absolute_avatar(user));
                alert
            })
            .collect();
        Ok(alerts)
    }

    /// Username owning the token, the one "assign to me" assigns.
    async fn current_username(&self) -> Result<String, ProviderError> {
        self.graphql::<CurrentUserData>(CURRENT_USER_QUERY, json!({}))
            .await?
            .current_user
            .map(|user| user.username)
            .ok_or_else(|| {
                ProviderError::Config(
                    "GitLab token cannot read its own user: grant it user read access".to_string(),
                )
            })
    }

    /// Self-hosted instances return avatar paths relative to the instance.
    fn absolute_avatar(&self, user: &mut GitlabAssignee) {
        if let Some(avatar_url) = &mut user.avatar_url
            && avatar_url.starts_with('/')
        {
            *avatar_url = format!("{}{}", self.url, avatar_url);
        }
    }
}
