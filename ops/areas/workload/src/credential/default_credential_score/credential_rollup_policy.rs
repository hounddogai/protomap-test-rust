use super::*;

use acme_kit::numbers;

pub fn credential_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a credential rollup score falls in.
pub fn classify_credential_rollup(score: f64) -> String {
    if score < 58.3 {
        return "low".to_string();
    }
    if score < 71.0 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn credential_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
