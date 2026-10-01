//! This file contains the GitLab alert types, as returned by the GraphQL API.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use crate::entities::{
    alert::{AlertVariant, GitlabVariant, Severity},
    provider::ProviderAlert,
};

#[derive(Debug, Deserialize)]
pub struct GitlabAlert {
    /// Not part of the API response: set by the provider to build a unique id.
    #[serde(skip)]
    pub project_id: String,

    pub iid: String,
    pub title: Option<String>,
    pub severity: Option<String>,
    pub description: Option<String>,

    #[serde(rename = "startedAt")]
    pub started_at: Option<DateTime<Utc>>,

    #[serde(rename = "webUrl")]
    pub web_url: String,

    /// Payload received by GitLab, holding the Grafana alert JSON.
    pub details: Option<Value>,
}

impl ProviderAlert for GitlabAlert {
    fn id(&self) -> String {
        format!("gitlab-{}-{}", self.project_id, self.iid)
    }

    /// Grafana alert name when GitLab got the alert from Grafana, so
    /// unmatched GitLab rows read like Grafana ones.
    fn title(&self) -> Option<String> {
        self.detail("labels.alertname")
            .or_else(|| self.title.clone())
    }

    fn severity(&self) -> Option<Severity> {
        let severity = self.severity.as_ref()?.to_lowercase();
        Some(Severity::new(severity))
    }

    fn description(&self) -> Option<String> {
        self.detail("annotations.description")
            .or_else(|| self.description.clone())
    }

    fn summary(&self) -> Option<String> {
        self.detail("annotations.summary")
            .or_else(|| self.title.clone())
    }

    fn instance(&self) -> Option<String> {
        self.detail("labels.instance")
    }

    fn starts_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    fn variant(&self) -> AlertVariant {
        AlertVariant::Gitlab(GitlabVariant {
            iid: self.iid.clone(),
            web_url: self.web_url.clone(),
            fingerprints: self
                .details
                .as_ref()
                .map(extract_fingerprints)
                .unwrap_or_default(),
        })
    }
}

impl GitlabAlert {
    /// String value of a details key. GitLab flattens the payload it received
    /// (`labels.instance`), but nested objects are accepted too.
    fn detail(&self, key: &str) -> Option<String> {
        let details = self.details.as_ref()?;
        let value = details
            .get(key)
            .or_else(|| details.pointer(&format!("/{}", key.replace('.', "/"))))?;
        value.as_str().map(String::from)
    }
}

/// Collect every Grafana fingerprint found in GitLab alert details.
///
/// Handles a single Grafana alert, a grouped webhook payload
/// (`alerts[].fingerprint`), GitLab's flattened keys (`foo.fingerprint`) and
/// details stored as a JSON-encoded string.
pub fn extract_fingerprints(details: &Value) -> Vec<String> {
    let mut fingerprints = Vec::new();
    collect_fingerprints(details, &mut fingerprints);
    fingerprints
}

fn collect_fingerprints(value: &Value, fingerprints: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let is_fingerprint_key = key == "fingerprint" || key.ends_with(".fingerprint");
                match value {
                    Value::String(s) if is_fingerprint_key => {
                        if !fingerprints.contains(s) {
                            fingerprints.push(s.clone());
                        }
                    }
                    _ => collect_fingerprints(value, fingerprints),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_fingerprints(item, fingerprints);
            }
        }
        Value::String(s) => {
            // Only strings that look like JSON documents are worth parsing.
            if s.trim_start().starts_with(['{', '['])
                && let Ok(parsed) = serde_json::from_str::<Value>(s)
            {
                collect_fingerprints(&parsed, fingerprints);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{GitlabAlert, extract_fingerprints};
    use crate::entities::provider::{ProviderAlert, convert_alert};
    use serde_json::json;

    /// Shape of an alert sent by Grafana to a GitLab HTTP integration.
    fn grafana_forwarded_alert() -> GitlabAlert {
        serde_json::from_value(json!({
            "iid": "83578",
            "title": "SWAP usage of 81% on host1",
            "severity": "MEDIUM",
            "description": "Add more RAM.",
            "startedAt": "2026-10-01T20:23:20Z",
            "webUrl": "https://gitlab.com/group/project/-/alert_management/83578/details",
            "details": {
                "endsAt": "0001-01-01T00:00:00Z",
                "labels.alertname": "SWAP usage too high",
                "labels.severity": "medium",
                "labels.instance": "host1",
                "status": "firing",
                "startsAt": "2026-10-01T20:23:20Z",
                "annotations.__values__": "{\"A\":81.2,\"B\":81.2,\"C\":1}",
                "annotations.description": "Add more RAM.",
                "annotations.summary": "SWAP usage of 81% on host1",
                "fingerprint": "c9c47fa31dcc3751",
                "generatorURL": "https://grafana.example.com/alerting/grafana/a6232484/view"
            }
        }))
        .unwrap()
    }

    #[test]
    fn test_grafana_forwarded_alert() {
        let mut gitlab_alert = grafana_forwarded_alert();
        gitlab_alert.project_id = "42".to_string();
        let alert = convert_alert(&gitlab_alert);

        assert_eq!(alert.id, "gitlab-42-83578");
        assert_eq!(alert.title, "SWAP usage too high");
        assert_eq!(alert.severity.machinename, "medium");
        assert_eq!(alert.summary, "SWAP usage of 81% on host1");
        assert_eq!(alert.description, "Add more RAM.");
        assert_eq!(alert.instance, "host1");
        assert!(alert.starts_at.is_some());
        assert_eq!(
            alert.variants[0].match_keys(),
            vec!["c9c47fa31dcc3751".to_string()]
        );
        assert_eq!(
            alert.variants[0].link(),
            Some("https://gitlab.com/group/project/-/alert_management/83578/details")
        );
    }

    #[test]
    fn test_alert_without_grafana_details() {
        let gitlab_alert: GitlabAlert = serde_json::from_value(json!({
            "iid": "1",
            "title": "Manual alert",
            "severity": null,
            "description": null,
            "startedAt": null,
            "webUrl": "https://gitlab.com/group/project/-/alert_management/1/details",
            "details": null
        }))
        .unwrap();

        assert_eq!(gitlab_alert.title().as_deref(), Some("Manual alert"));
        assert_eq!(gitlab_alert.summary().as_deref(), Some("Manual alert"));
        assert!(gitlab_alert.instance().is_none());
        assert!(gitlab_alert.variant().match_keys().is_empty());
    }

    #[test]
    fn test_single_alert() {
        let details = json!({"labels": {"alertname": "Down"}, "fingerprint": "abc123"});
        assert_eq!(extract_fingerprints(&details), vec!["abc123"]);
    }

    #[test]
    fn test_grouped_payload() {
        let details = json!({
            "receiver": "gitlab",
            "alerts": [{"fingerprint": "aaa"}, {"fingerprint": "bbb"}, {"fingerprint": "aaa"}]
        });
        assert_eq!(extract_fingerprints(&details), vec!["aaa", "bbb"]);
    }

    #[test]
    fn test_flattened_keys() {
        let details = json!({"alert.fingerprint": "ccc", "alert.labels.severity": "high"});
        assert_eq!(extract_fingerprints(&details), vec!["ccc"]);
    }

    #[test]
    fn test_json_encoded_string() {
        let details = json!({"payload": "{\"alerts\":[{\"fingerprint\":\"ddd\"}]}"});
        assert_eq!(extract_fingerprints(&details), vec!["ddd"]);
    }

    #[test]
    fn test_no_fingerprint() {
        let details = json!({"title": "something", "count": 3});
        assert!(extract_fingerprints(&details).is_empty());
    }
}
