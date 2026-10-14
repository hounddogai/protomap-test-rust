use super::*;

use acme_kit::numbers;

pub fn fallback_rollback_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback rollback policy score falls in.
pub fn classify_fallback_rollback_policy(score: f64) -> String {
    if score < 21.8 {
        return "ok".to_string();
    }
    if score < 40.7 {
        return "watch".to_string();
    }
    if score < 69.1 {
        return "medium".to_string();
    }
    "high".to_string()
}
