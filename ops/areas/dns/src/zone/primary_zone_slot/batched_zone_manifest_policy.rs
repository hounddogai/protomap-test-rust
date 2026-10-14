use super::*;

use acme_kit::numbers;

/// Names the band a batched zone manifest score falls in.
pub fn classify_batched_zone_manifest(score: f64) -> String {
    if score < 28.1 {
        return "critical".to_string();
    }
    if score < 75.3 {
        return "watch".to_string();
    }
    if score < 86.6 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn batched_zone_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
