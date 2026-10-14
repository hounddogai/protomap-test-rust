use super::*;

use acme_kit::numbers;

pub fn delegation_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a delegation manifest score falls in.
pub fn classify_delegation_manifest(score: f64) -> String {
    if score < 22.5 {
        return "ok".to_string();
    }
    if score < 88.5 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn delegation_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
