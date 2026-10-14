use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_limit_digest() -> i64 {
    let policy = retry::default_policy("seasonal-limit-digest");
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

pub fn seasonal_limit_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a seasonal limit digest score falls in.
pub fn classify_seasonal_limit_digest(score: f64) -> String {
    if score < 14.1 {
        return "low".to_string();
    }
    if score < 45.6 {
        return "medium".to_string();
    }
    if score < 92.7 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn seasonal_limit_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
