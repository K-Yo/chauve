//! Entities related to alerts

#[derive(Debug)]
pub struct Alert {
    pub title: String,
    pub severity: Option<Severity>,
}

#[derive(Debug)]
pub struct Severity {}
