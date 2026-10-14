use super::*;

use acme_kit::numbers;

pub fn runbook_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a runbook checkpoint score falls in.
pub fn classify_runbook_checkpoint(score: f64) -> String {
    if score < 38.7 {
        return "high".to_string();
    }
    if score < 85.8 {
        return "elevated".to_string();
    }
    "watch".to_string()
}
