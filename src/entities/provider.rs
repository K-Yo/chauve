use crate::entities::alert::{Alert, Severity};
use async_trait::async_trait;

/// An alert for a provider
pub trait ProviderAlert {
    fn title(&self) -> Option<String> {
        None
    }
    fn severity(&self) -> Option<Severity> {
        None
    }
    fn link(&self) -> Option<String> {
        None
    }
}

pub fn convert_alert<T: ProviderAlert>(provider_alert: &T) -> Alert {
    Alert {
        title: provider_alert.title().unwrap_or_default(),
        severity: provider_alert.severity().unwrap_or_default(),
        link: provider_alert.link().unwrap_or_default()
    }
}

/// A provider
#[async_trait]
pub trait Provider {

    /// Retrieve normalized alerts from the provider.
    async fn alerts(&self) -> Vec<Alert>;

    // needed to make it clonable in a box
    fn clone_box(&self) -> Box<dyn Provider>;
}


impl Clone for Box<dyn Provider> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}