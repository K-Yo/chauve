//! Entities related to alerts

use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub description: String,
    pub summary: String,
    pub link: String,
}


fn to_machinename(input: &str) -> String {
    input.chars()
         .filter(|c| c.is_ascii() && ! c.is_whitespace())
         .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Severity {
    pub machinename: String,
    pub icon: String,
}

impl Default for Severity {
    fn default() -> Severity {
        Severity {
            machinename: String::from("unknown"),
            icon: String::from("❔")
        }
    }
}

/// Returns a numeric order for a severity name (lower = more severe).
/// critical=0, high=1, medium=2, low=3, unknown/other=4
pub fn severity_order(name: &str) -> u8 {
    match name {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

impl Severity {
    pub fn new(sev: String) -> Severity {
        Severity {
            machinename: to_machinename(&sev),
            icon: String::from("⛔")
        }
    }

    /// Numeric sort order: lower value = more severe.
    pub fn order(&self) -> u8 {
        severity_order(&self.machinename)
    }
}
