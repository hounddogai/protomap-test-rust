use super::*;

use acme_kit::numbers;

pub fn binding_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn binding_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a binding snapshot score falls in.
pub fn classify_binding_snapshot(score: f64) -> String {
    if score < 67.8 {
        return "high".to_string();
    }
    if score < 92.7 {
        return "medium".to_string();
    }
    if score < 95.4 {
        return "ok".to_string();
    }
    "watch".to_string()
}
