use super::*;

use acme_kit::numbers;

pub fn fallback_overage_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a fallback overage rule score falls in.
pub fn classify_fallback_overage_rule(score: f64) -> String {
    if score < 33.4 {
        return "minimal".to_string();
    }
    if score < 35.8 {
        return "high".to_string();
    }
    if score < 37.9 {
        return "elevated".to_string();
    }
    if score < 66.8 {
        return "low".to_string();
    }
    "medium".to_string()
}
