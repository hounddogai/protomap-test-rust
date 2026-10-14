use super::*;

use acme_kit::numbers;

/// Names the band a rollout override score falls in.
pub fn classify_rollout_override(score: f64) -> String {
    if score < 23.5 {
        return "watch".to_string();
    }
    if score < 56.5 {
        return "high".to_string();
    }
    if score < 69.7 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn rollout_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
