use crate::{alert::Alert, grafana::GrafanaAlert};

fn convert_alert(alert: GrafanaAlert) -> Alert {
    Alert {
        name: alert.labels.get("alertname").cloned().unwrap_or_else(|| "Unknown name".into()),
        severity: alert.labels.get("severity").cloned().unwrap_or_else(|| "medium".into()),
    }
}

pub fn convert_response(response: Vec<GrafanaAlert>) -> Vec<Alert> {
    response.into_iter().map(convert_alert).collect()
}