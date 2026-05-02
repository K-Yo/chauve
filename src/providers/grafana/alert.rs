//! This file contains the code to communicate with Grafana API.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use crate::entities::{alert::Severity, provider::ProviderAlert};

#[derive(Debug, Deserialize)]
pub struct GrafanaAlert {
    pub annotations: Option<HashMap<String, String>>,
    pub labels: HashMap<String, String>,

    #[serde(rename = "startsAt")]
    pub starts_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    #[serde(default, rename = "endsAt")]
    pub ends_at: Option<DateTime<Utc>>,

    pub status: AlertStatus,

    pub fingerprint: String,

    #[serde(rename = "generatorURL")]
    pub generator_url: Option<String>,
}

impl ProviderAlert for GrafanaAlert {
    fn id(&self) -> String {
        self.fingerprint.clone()
    }
    fn severity(&self) -> Option<Severity> {
        let Some(severity_str) = self.labels.get("severity") else {
            return None;
        };

        Some(Severity::new(severity_str.to_string()))
    }

    fn title(&self) -> Option<String> {
        self.labels.get("alertname").cloned()
    }

    fn link(&self) -> Option<String> {
        self.generator_url.clone()
    }

    fn description(&self) -> Option<String> {
        self.annotations.clone()?.get("description").cloned()
    }

    fn summary(&self) -> Option<String> {
        self.annotations.clone()?.get("summary").cloned()
    }
}

#[derive(Debug, Deserialize)]
pub struct AlertStatus {
    pub state: String,
    #[serde(default, rename = "silencedBy")]
    pub silenced_by: Vec<String>,
    #[serde(default, rename = "inhibitedBy")]
    pub inhibited_by: Vec<String>,
}

impl AlertStatus {
    pub fn is_suppressed(&self) -> bool {
        self.state == "suppressed" || !self.silenced_by.is_empty() || !self.inhibited_by.is_empty()
    }
}
