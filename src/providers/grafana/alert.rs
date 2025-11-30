//! This file contains the code to communicate with Grafana API.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{entities::alert::Severity, provider::ProviderAlert};

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
    pub receivers: Vec<Receiver>,

    pub fingerprint: String,

    #[serde(rename = "generatorURL")]
    pub generator_url: Option<String>,
}

impl ProviderAlert for GrafanaAlert {
    fn severity(&self) -> Option<Severity> {
        todo!()
    }
    fn title(&self) -> String {
        todo!()
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

#[derive(Debug, Deserialize)]
pub struct Receiver {
    pub name: String,
}

pub type ApiResponse = Vec<GrafanaAlert>;

pub struct Grafana {}
