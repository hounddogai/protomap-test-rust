use super::*;

use acme_kit::numbers;

pub fn runbook_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a runbook group score falls in.
pub fn classify_runbook_group(score: f64) -> String {
    if score < 18.3 {
        return "watch".to_string();
    }
    if score < 48.8 {
        return "critical".to_string();
    }
    if score < 89.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
