// 2. Consider Error Handling for Failed Provider Calls
// Right now, failed providers just return vec![], which hides whether it was intentional or due to I/O error.
// Could enhance Provider::alerts() return type:
// async fn alerts(&self) -> Result<Vec<Alert>, ProviderError>;
// Then poll() could log diagnostics or expose provider health.

// 3. Add Logging
// Use tracing or log! macros to record:
// Start/end of poll
// Number of alerts per provider
// Errors (if enhanced above)

// 4. Caching Strategy (Optional)
// If polling is frequent and providers rate-limited, cache results with TTL.

// 5. Graceful Shutdown / Cancellation
// Not a concern in test, but in production, ensure futures aren’t dropped mid-call without cleanup.

// 6. Improve Poller::new() Scalability
// Currently hardcodes Grafana only.
// Suggestion: refactor settings to allow iterating over multiple provider kinds.

use chrono::{DateTime, Utc};
use futures::future::join_all;
use std::fmt;
use tracing::{debug, warn};

use crate::{
    entities::{alert::Alert, provider::Provider, settings::Settings},
    providers::grafana::provider::GrafanaProvider,
};

#[derive(Debug, Clone)]
pub struct PollStats {
    pub success_count: usize,
    pub error_count: usize,
    pub total_providers: usize,
    pub errors: Vec<String>,
}

impl fmt::Display for PollStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} providers succeeded, {} failed out of {}",
            self.success_count, self.error_count, self.total_providers
        )
    }
}

#[derive(Clone)]
pub struct PollerData {
    pub alerts: Vec<Alert>,
    pub last_poll_time: DateTime<Utc>,
    pub stats: PollStats,
}

#[derive(Clone)]
pub struct Poller {
    last_poll_time: Option<DateTime<Utc>>,
    providers: Vec<Box<dyn Provider>>,
    cached_alerts: Vec<Alert>,
}

impl Poller {
    /// poll for new alerts, and return the result
    ///
    /// polling stores internally alerts for future access (using alerts())
    /// this should be scheduled regularly
    pub async fn poll_once(&self) -> Result<PollerData, PollStats> {
        let now = Utc::now();
        debug!(target: "poller", "Starting poll at {}", now.to_rfc3339());

        let futures: Vec<_> = self.providers.iter().map(|p| p.alerts()).collect();
        let results = join_all(futures).await;

        let mut all_alerts = Vec::new();
        let mut errors = Vec::new();
        let mut success_count = 0;

        let total = results.len();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(alerts) => {
                    success_count += 1;
                    debug!(target: "poller", "Provider {} returned {} alert(s)", i, alerts.len());
                    all_alerts.extend(alerts);
                }
                Err(e) => {
                    warn!(target: "poller", "Provider {} failed: {}", i, e);
                    errors.push(format!("Provider #{}: {}", i, e));
                }
            }
        }

        let error_count = total - success_count;

        let stats = PollStats {
            success_count,
            error_count,
            total_providers: total,
            errors,
        };

        let data = PollerData {
            alerts: all_alerts,
            last_poll_time: now,
            stats: stats.clone(),
        };

        if error_count > 0 {
            warn!(target: "poller", "Poll completed with issues: {}", stats);
        }

        Ok(data)
    }

    pub fn update_with(&mut self, data: PollerData) {
        self.cached_alerts = data.alerts;
        self.last_poll_time = Some(data.last_poll_time);
    }

    pub fn last_poll_time(&self) -> String {
        match self.last_poll_time {
            Some(date) => date.to_rfc3339(),
            _ => "Never".to_string(),
        }
    }

    /// return alert list
    ///
    /// this will not poll for new alerts, use poll() for that.
    pub fn alerts(&self) -> Vec<Alert> {
        self.cached_alerts.clone()
    }

    pub fn new(settings: Settings) -> Poller {
        // TODO: extend into a full builder instead of simple factory and avoid having provider types appear here.
        let mut all_providers: Vec<Box<dyn Provider>> = vec![];
        for grafana_settings in settings.providers.grafana {
            let provider = GrafanaProvider::new(grafana_settings.url, grafana_settings.token);
            all_providers.push(Box::new(provider));
        }
        Poller {
            last_poll_time: None,
            providers: all_providers,
            cached_alerts: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Add required test dependencies
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::entities::{
        alert::{Alert, Severity},
        provider::{Provider, ProviderError},
    };
    use tokio;

    impl Poller {
        /// poll function used in tests to get alerts in a single call
        pub async fn poll(&mut self) -> Result<Vec<Alert>, PollStats> {
            let result = self.poll_once().await?;
            let alerts = result.alerts.clone();
            self.update_with(result);
            Ok(alerts)
        }
    }

    // Mock provider for testing
    struct MockProvider {
        id: u8,
        should_error: bool,
        calls: Arc<Mutex<Vec<u8>>>,
    }

    impl MockProvider {
        fn new(id: u8, should_error: bool, calls: Arc<Mutex<Vec<u8>>>) -> Self {
            Self {
                id,
                should_error,
                calls,
            }
        }
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        async fn alerts(&self) -> Result<Vec<Alert>, ProviderError> {
            // Record that this provider was called
            self.calls.lock().unwrap().push(self.id);

            if self.should_error {
                // Return some error from reqwest - use a truly invalid URL
                Err(ProviderError::Anyhow(
                    reqwest::Url::parse("://invalid-url").unwrap_err().into(),
                ))
            } else {
                // Return some test alerts
                Ok(vec![
                    Alert {
                        id: format!("1"),
                        title: format!("Alert from provider {} - 1", self.id),
                        severity: Severity::new(format!("severity-{}", self.id)),
                        link: format!("http://provider{}/alert1", self.id),
                        description: "".to_string(),
                        summary: "".to_string(),
                        instance: "".to_string(),
                        starts_at: None,
                    },
                    Alert {
                        id: format!("2"),
                        title: format!("Alert from provider {} - 2", self.id),
                        severity: Severity::new(format!("severity-{}", self.id)),
                        link: format!("http://provider{}/alert2", self.id),
                        description: "".to_string(),
                        summary: "".to_string(),
                        instance: "".to_string(),
                        starts_at: Some(Utc::now() - chrono::Duration::hours(2)),
                    },
                ])
            }
        }

        fn clone_box(&self) -> Box<dyn Provider> {
            Box::new(Self {
                id: self.id,
                should_error: self.should_error,
                calls: Arc::clone(&self.calls),
            })
        }
    }

    #[tokio::test]
    async fn test_poller_single_provider() {
        // Setup: create a poller with one provider
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = MockProvider::new(1, false, Arc::clone(&calls));
        let mut poller = Poller {
            last_poll_time: None,
            providers: vec![Box::new(provider)],
            cached_alerts: Vec::new(),
        };

        // When: polling the poller
        let result = poller.poll().await;
        let alerts = result.expect("Poll should succeed");

        // Then: should have called the provider and returned its alerts
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert_eq!(calls.lock().unwrap()[0], 1);
        assert_eq!(alerts.len(), 2);
        assert!(alerts[0].title.contains("provider 1"));
        assert!(alerts[1].title.contains("provider 1"));

        // Check last poll time was updated
        assert_ne!(poller.last_poll_time(), "Never");
    }

    #[tokio::test]
    async fn test_poller_multiple_providers() {
        // Setup: create a poller with multiple providers
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider1 = MockProvider::new(1, false, Arc::clone(&calls));
        let provider2 = MockProvider::new(2, false, Arc::clone(&calls));
        let provider3 = MockProvider::new(3, false, Arc::clone(&calls));

        let mut poller = Poller {
            last_poll_time: None,
            providers: vec![
                Box::new(provider1),
                Box::new(provider2),
                Box::new(provider3),
            ],
            cached_alerts: Vec::new(),
        };

        // When: polling the poller
        let result = poller.poll().await;
        let alerts = result.expect("Poll should succeed");

        // Then: should have called all providers and returned all alerts
        let call_list = calls.lock().unwrap();
        assert_eq!(call_list.len(), 3);
        assert!(call_list.contains(&1));
        assert!(call_list.contains(&2));
        assert!(call_list.contains(&3));

        // Should have 2 alerts from each of 3 providers = 6 alerts
        assert_eq!(alerts.len(), 6);

        // Check providers are still in the list for next poll
        drop(call_list);

        // When: polling again
        let result = poller.poll().await;
        result.expect("Second poll should succeed");

        // Then: should have called all providers again
        let call_list = calls.lock().unwrap();
        // Each provider should have been called twice
        assert_eq!(call_list.iter().filter(|&&x| x == 1).count(), 2);
        assert_eq!(call_list.iter().filter(|&&x| x == 2).count(), 2);
        assert_eq!(call_list.iter().filter(|&&x| x == 3).count(), 2);
    }

    #[tokio::test]
    async fn test_poller_with_failing_provider() {
        // Setup: create a poller with a mix of working and failing providers
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider1 = MockProvider::new(1, false, Arc::clone(&calls)); // working
        let provider2 = MockProvider::new(2, true, Arc::clone(&calls)); // failing
        let provider3 = MockProvider::new(3, false, Arc::clone(&calls)); // working

        let mut poller = Poller {
            last_poll_time: None,
            providers: vec![
                Box::new(provider1),
                Box::new(provider2),
                Box::new(provider3),
            ],
            cached_alerts: Vec::new(),
        };

        // When: polling the poller
        let result = poller.poll().await;

        // Then: should return Ok with alerts from working providers, even if some providers failed
        let alerts = result.expect("Poll should return alerts when some providers succeed");

        let call_list = calls.lock().unwrap();
        assert_eq!(call_list.len(), 3);
        assert!(call_list.contains(&1));
        assert!(call_list.contains(&2));
        assert!(call_list.contains(&3));

        // Should have 2 alerts from provider 1 and 2 from provider 3, none from provider 2
        assert_eq!(alerts.len(), 4);

        // Alerts should be from providers 1 and 3 only
        for alert in alerts {
            assert!(!alert.title.contains("provider 2"));
        }
    }

    #[tokio::test]
    async fn test_poller_empty_providers() {
        // Setup: create a poller with no providers
        let mut poller = Poller {
            last_poll_time: None,
            providers: vec![],
            cached_alerts: Vec::new(),
        };

        // When: polling the poller
        let result = poller.poll().await;
        let alerts = result.expect("Poll with no providers should succeed");

        // Then: should return empty vec but not panic
        assert_eq!(alerts.len(), 0);

        // Try polling multiple times to ensure stability
        for _ in 0..3 {
            let result = poller.poll().await;
            let more_alerts = result.expect("Subsequent polls should succeed");
            assert_eq!(more_alerts.len(), 0);
        }
    }
}
