use super::*;

use acme_kit::numbers;

/// Names the band a ttl policy audit score falls in.
pub fn classify_ttl_policy_audit(score: f64) -> String {
    if score < 18.2 {
        return "watch".to_string();
    }
    if score < 51.2 {
        return "high".to_string();
    }
    if score < 76.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn ttl_policy_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.83;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
