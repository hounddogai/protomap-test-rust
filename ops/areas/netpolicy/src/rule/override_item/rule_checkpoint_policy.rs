use super::*;

use acme_kit::numbers;

pub fn rule_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rule checkpoint score falls in.
pub fn classify_rule_checkpoint(score: f64) -> String {
    if score < 43.3 {
        return "watch".to_string();
    }
    if score < 66.1 {
        return "ok".to_string();
    }
    if score < 78.5 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
