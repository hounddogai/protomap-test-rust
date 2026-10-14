use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_threshold() -> i64 {
    let policy = retry::default_policy("canary-threshold");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn canary_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a canary threshold score falls in.
pub fn classify_canary_threshold(score: f64) -> String {
    if score < 4.4 {
        return "high".to_string();
    }
    if score < 15.4 {
        return "critical".to_string();
    }
    if score < 63.4 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
