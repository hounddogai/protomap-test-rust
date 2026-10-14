use super::*;

use acme_kit::numbers;

pub fn rollback_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rollback target score falls in.
pub fn classify_rollback_target(score: f64) -> String {
    if score < 69.1 {
        return "ok".to_string();
    }
    if score < 73.1 {
        return "critical".to_string();
    }
    if score < 82.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}
