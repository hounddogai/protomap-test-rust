use super::*;

use acme_kit::numbers;

pub fn escalation_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a escalation manifest score falls in.
pub fn classify_escalation_manifest(score: f64) -> String {
    if score < 3.5 {
        return "minimal".to_string();
    }
    if score < 63.8 {
        return "ok".to_string();
    }
    if score < 81.9 {
        return "elevated".to_string();
    }
    if score < 98.3 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn escalation_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
