use crate::entities::alert::{Alert, Severity};
use async_trait::async_trait;

/// An alert for a provider
pub trait ProviderAlert {
    fn title(&self) -> String;
    fn severity(&self) -> Option<Severity>;
}

fn convert_alert<T: ProviderAlert>(provider_alert: &T) -> Alert {
    Alert {
        title: provider_alert.title(),
        severity: provider_alert.severity(),
    }
}

/// A provider
#[async_trait]
pub trait Provider<T: ProviderAlert> {
    /// Pull provider alerts from the API.
    async fn pull(&self) -> Vec<T>;

    /// Retrieve normalized alerts from the provider.
    async fn alerts(&self) -> Vec<Alert> {
        let provider_alerts = self.pull().await;
        provider_alerts.iter().map(convert_alert).collect()
    }
}
