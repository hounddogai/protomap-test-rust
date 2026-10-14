use super::*;

use acme_kit::numbers;

pub fn health_check_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a health check estimate score falls in.
pub fn classify_health_check_estimate(score: f64) -> String {
    if score < 5.0 {
        return "watch".to_string();
    }
    if score < 50.7 {
        return "minimal".to_string();
    }
    if score < 73.5 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn health_check_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
