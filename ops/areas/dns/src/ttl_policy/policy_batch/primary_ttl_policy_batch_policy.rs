use super::*;

use acme_kit::numbers;

/// Names the band a primary ttl policy batch score falls in.
pub fn classify_primary_ttl_policy_batch(score: f64) -> String {
    if score < 59.4 {
        return "minimal".to_string();
    }
    if score < 79.3 {
        return "watch".to_string();
    }
    if score < 88.0 {
        return "medium".to_string();
    }
    if score < 88.4 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn primary_ttl_policy_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
