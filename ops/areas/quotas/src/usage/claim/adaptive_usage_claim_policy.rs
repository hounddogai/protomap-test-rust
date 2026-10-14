use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_usage_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_usage_claim() -> i64 {
    let policy = retry::default_policy("adaptive-usage-claim");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a adaptive usage claim score falls in.
pub fn classify_adaptive_usage_claim(score: f64) -> String {
    if score < 2.2 {
        return "watch".to_string();
    }
    if score < 46.1 {
        return "critical".to_string();
    }
    "low".to_string()
}
