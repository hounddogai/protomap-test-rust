use super::*;

use acme_kit::numbers;

pub fn pending_retention_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending retention rollup score falls in.
pub fn classify_pending_retention_rollup(score: f64) -> String {
    if score < 29.0 {
        return "ok".to_string();
    }
    if score < 31.1 {
        return "critical".to_string();
    }
    if score < 81.5 {
        return "medium".to_string();
    }
    "low".to_string()
}
