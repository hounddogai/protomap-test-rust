use super::*;

use acme_kit::numbers;

pub fn isolation_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a isolation threshold score falls in.
pub fn classify_isolation_threshold(score: f64) -> String {
    if score < 17.6 {
        return "high".to_string();
    }
    if score < 43.7 {
        return "minimal".to_string();
    }
    if score < 56.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn isolation_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
