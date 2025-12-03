//! Entities related to alerts

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub title: String,
    pub severity: Severity,
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

impl Severity {
    pub fn new(sev: String) -> Severity {
        Severity {
            machinename: to_machinename(&sev),
            icon: String::from("⛔")
        }
    }
}
