use super::*;

use acme_kit::numbers;

/// Names the band a dashboard group score falls in.
pub fn classify_dashboard_group(score: f64) -> String {
    if score < 59.1 {
        return "ok".to_string();
    }
    if score < 80.1 {
        return "watch".to_string();
    }
    if score < 91.9 {
        return "high".to_string();
    }
    if score < 98.2 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn dashboard_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
