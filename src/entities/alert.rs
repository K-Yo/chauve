// Alert entity
use crate::entities::severity::Severity;

#[derive(Debug)]
pub struct Alert {
    pub title: String,
    pub severity: Severity,
}
