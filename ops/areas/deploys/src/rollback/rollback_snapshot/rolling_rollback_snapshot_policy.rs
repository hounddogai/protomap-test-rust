use super::*;

use acme_kit::numbers;

pub fn rolling_rollback_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling rollback snapshot score falls in.
pub fn classify_rolling_rollback_snapshot(score: f64) -> String {
    if score < 38.1 {
        return "critical".to_string();
    }
    if score < 67.5 {
        return "elevated".to_string();
    }
    "watch".to_string()
}
