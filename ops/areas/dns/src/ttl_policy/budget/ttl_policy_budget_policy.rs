use super::*;

use acme_kit::numbers;

/// Names the band a ttl policy budget score falls in.
pub fn classify_ttl_policy_budget(score: f64) -> String {
    if score < 39.8 {
        return "medium".to_string();
    }
    if score < 86.0 {
        return "critical".to_string();
    }
    if score < 89.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn ttl_policy_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
