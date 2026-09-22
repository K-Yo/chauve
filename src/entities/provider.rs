use crate::entities::alert::{Alert, Severity};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// An alert for a provider
pub trait ProviderAlert {
    fn id(&self) -> String {
        "".into()
    }
    fn title(&self) -> Option<String> {
        None
    }
    fn severity(&self) -> Option<Severity> {
        None
    }
    fn link(&self) -> Option<String> {
        None
    }
    fn description(&self) -> Option<String> {
        None
    }
    fn summary(&self) -> Option<String> {
        None
    }
    fn instance(&self) -> Option<String> {
        None
    }
    /// When the alert started firing, if the provider reports it.
    fn starts_at(&self) -> Option<DateTime<Utc>> {
        None
    }
}

pub fn convert_alert<T: ProviderAlert>(provider_alert: &T) -> Alert {
    Alert {
        id: provider_alert.id(),

        title: provider_alert.title().unwrap_or_default(),
        severity: provider_alert.severity().unwrap_or_default(),
        link: provider_alert.link().unwrap_or_default(),
        description: provider_alert.description().unwrap_or_default(),
        summary: provider_alert.summary().unwrap_or_default(),
        instance: provider_alert.instance().unwrap_or_default(),
        starts_at: provider_alert.starts_at(),
    }
}

/// A provider
///
/// Can be grafana, prometheus, or any other system that can provide alerts.
#[async_trait]
pub trait Provider {
    /// Retrieve normalized alerts from the provider.
    async fn alerts(&self) -> Result<Vec<Alert>, ProviderError>;

    // needed to make it clonable in a box
    fn clone_box(&self) -> Box<dyn Provider>;
}

// Define a custom error type (optional, but recommended)
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("unknown provider error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl Clone for Box<dyn Provider> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
