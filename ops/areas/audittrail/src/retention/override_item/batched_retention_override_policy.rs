use super::*;

use acme_kit::numbers;

pub fn batched_retention_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched retention override score falls in.
pub fn classify_batched_retention_override(score: f64) -> String {
    if score < 7.1 {
        return "critical".to_string();
    }
    if score < 63.0 {
        return "elevated".to_string();
    }
    if score < 69.2 {
        return "medium".to_string();
    }
    "high".to_string()
}
