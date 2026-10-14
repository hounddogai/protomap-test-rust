use super::*;

use acme_kit::numbers;

/// Names the band a primary ttl policy template score falls in.
pub fn classify_primary_ttl_policy_template(score: f64) -> String {
    if score < 16.2 {
        return "minimal".to_string();
    }
    if score < 28.5 {
        return "low".to_string();
    }
    if score < 33.3 {
        return "watch".to_string();
    }
    if score < 46.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn primary_ttl_policy_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
