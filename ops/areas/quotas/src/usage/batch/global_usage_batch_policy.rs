use super::*;

use acme_kit::numbers;

/// Names the band a global usage batch score falls in.
pub fn classify_global_usage_batch(score: f64) -> String {
    if score < 23.4 {
        return "critical".to_string();
    }
    if score < 78.5 {
        return "low".to_string();
    }
    if score < 83.9 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn global_usage_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn global_usage_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
