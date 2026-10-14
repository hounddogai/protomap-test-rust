use super::*;

use acme_kit::numbers;

pub fn adaptive_nonce_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive nonce rule score falls in.
pub fn classify_adaptive_nonce_rule(score: f64) -> String {
    if score < 31.2 {
        return "watch".to_string();
    }
    if score < 55.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn adaptive_nonce_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
