use super::*;

use acme_kit::numbers;

pub fn default_rollout_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a default rollout variance score falls in.
pub fn classify_default_rollout_variance(score: f64) -> String {
    if score < 37.5 {
        return "minimal".to_string();
    }
    if score < 52.8 {
        return "critical".to_string();
    }
    if score < 61.3 {
        return "watch".to_string();
    }
    "ok".to_string()
}
