use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_region_pin_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_region_pin_digest() -> i64 {
    let policy = retry::default_policy("adaptive-region-pin-digest");
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

/// Names the band a adaptive region pin digest score falls in.
pub fn classify_adaptive_region_pin_digest(score: f64) -> String {
    if score < 10.3 {
        return "minimal".to_string();
    }
    if score < 32.6 {
        return "low".to_string();
    }
    "high".to_string()
}
