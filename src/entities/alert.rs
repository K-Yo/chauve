//! Entities related to alerts

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub description: String,
    pub summary: String,
    pub link: String,
    pub instance: String,
    /// When the alert started firing, if the provider reports it.
    pub starts_at: Option<DateTime<Utc>>,
}

/// Compact two-unit relative duration: "45s", "12m", "2h 13m", "3d 4h".
/// Sub-second and future timestamps render as "0s".
pub fn format_age(since: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = now.signed_duration_since(since).num_seconds().max(0);

    let minutes = seconds / 60;
    if minutes == 0 {
        return format!("{}s", seconds);
    }

    let hours = minutes / 60;
    if hours == 0 {
        return format!("{}m", minutes);
    }

    let days = hours / 24;
    if days == 0 {
        return match minutes % 60 {
            0 => format!("{}h", hours),
            m => format!("{}h {}m", hours, m),
        };
    }

    match hours % 24 {
        0 => format!("{}d", days),
        h => format!("{}d {}h", days, h),
    }
}

fn to_machinename(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii() && !c.is_whitespace())
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
            icon: String::from("❔"),
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
            icon: String::from("⛔"),
        }
    }

    /// Numeric sort order: lower value = more severe.
    pub fn order(&self) -> u8 {
        severity_order(&self.machinename)
    }
}

#[cfg(test)]
mod tests {
    use super::format_age;
    use chrono::{Duration, Utc};

    /// Format the age of an alert that started `ago` before a fixed "now".
    fn age(ago: Duration) -> String {
        let now = Utc::now();
        format_age(now - ago, now)
    }

    #[test]
    fn test_format_age_seconds() {
        assert_eq!(age(Duration::zero()), "0s");
        assert_eq!(age(Duration::seconds(45)), "45s");
        assert_eq!(age(Duration::seconds(59)), "59s");
    }

    #[test]
    fn test_format_age_minutes() {
        assert_eq!(age(Duration::seconds(60)), "1m");
        assert_eq!(age(Duration::minutes(12)), "12m");
        assert_eq!(age(Duration::minutes(59)), "59m");
    }

    #[test]
    fn test_format_age_hours() {
        assert_eq!(age(Duration::minutes(133)), "2h 13m");
        assert_eq!(age(Duration::hours(23) + Duration::minutes(59)), "23h 59m");
    }

    #[test]
    fn test_format_age_days() {
        assert_eq!(age(Duration::hours(76)), "3d 4h");
        assert_eq!(age(Duration::days(30)), "30d");
    }

    #[test]
    fn test_format_age_trims_zero_remainder() {
        assert_eq!(age(Duration::hours(2)), "2h");
        assert_eq!(age(Duration::days(3)), "3d");
    }

    #[test]
    fn test_format_age_clamps_future_timestamps() {
        let now = Utc::now();
        assert_eq!(format_age(now + Duration::hours(1), now), "0s");
    }
}
