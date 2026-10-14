use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_token_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_token_target() -> i64 {
    let policy = retry::default_policy("fallback-token-target");
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

/// Names the band a fallback token target score falls in.
pub fn classify_fallback_token_target(score: f64) -> String {
    if score < 30.3 {
        return "critical".to_string();
    }
    if score < 40.8 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
