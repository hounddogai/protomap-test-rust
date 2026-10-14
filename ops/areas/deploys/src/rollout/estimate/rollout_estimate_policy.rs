use super::*;

use acme_kit::numbers;

/// Names the band a rollout estimate score falls in.
pub fn classify_rollout_estimate(score: f64) -> String {
    if score < 56.4 {
        return "high".to_string();
    }
    if score < 85.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn rollout_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
