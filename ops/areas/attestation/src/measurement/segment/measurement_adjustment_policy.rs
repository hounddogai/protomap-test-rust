use super::*;

use acme_kit::numbers;

pub fn measurement_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a measurement adjustment score falls in.
pub fn classify_measurement_adjustment(score: f64) -> String {
    if score < 56.4 {
        return "high".to_string();
    }
    if score < 69.1 {
        return "watch".to_string();
    }
    if score < 75.4 {
        return "elevated".to_string();
    }
    if score < 77.8 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn measurement_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
