use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a silence bundle score falls in.
pub fn classify_silence_bundle(score: f64) -> String {
    if score < 31.2 {
        return "low".to_string();
    }
    if score < 75.1 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn silence_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_bundle() -> i64 {
    let policy = retry::default_policy("silence-bundle");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
