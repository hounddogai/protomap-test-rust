use super::*;

use acme_kit::numbers;

pub fn redaction_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a redaction policy score falls in.
pub fn classify_redaction_policy(score: f64) -> String {
    if score < 3.5 {
        return "minimal".to_string();
    }
    if score < 31.5 {
        return "low".to_string();
    }
    if score < 39.3 {
        return "high".to_string();
    }
    if score < 54.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}
