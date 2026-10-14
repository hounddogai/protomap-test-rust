use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a silence quota score falls in.
pub fn classify_silence_quota(score: f64) -> String {
    if score < 62.0 {
        return "critical".to_string();
    }
    if score < 92.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn silence_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.08;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_quota() -> i64 {
    let policy = retry::default_policy("silence-quota");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
