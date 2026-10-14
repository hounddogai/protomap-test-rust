use super::*;

use acme_kit::numbers;

pub fn strict_identity_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.06;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a strict identity target score falls in.
pub fn classify_strict_identity_target(score: f64) -> String {
    if score < 41.6 {
        return "ok".to_string();
    }
    if score < 83.4 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
