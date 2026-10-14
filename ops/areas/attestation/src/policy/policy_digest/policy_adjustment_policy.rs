use super::*;

use acme_kit::numbers;

pub fn policy_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a policy adjustment score falls in.
pub fn classify_policy_adjustment(score: f64) -> String {
    if score < 24.8 {
        return "critical".to_string();
    }
    if score < 57.7 {
        return "watch".to_string();
    }
    if score < 72.5 {
        return "ok".to_string();
    }
    if score < 82.0 {
        return "minimal".to_string();
    }
    "high".to_string()
}
