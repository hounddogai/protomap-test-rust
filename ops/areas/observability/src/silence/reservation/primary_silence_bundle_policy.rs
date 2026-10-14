use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a primary silence bundle score falls in.
pub fn classify_primary_silence_bundle(score: f64) -> String {
    if score < 26.1 {
        return "minimal".to_string();
    }
    if score < 53.6 {
        return "ok".to_string();
    }
    if score < 58.5 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_silence_bundle() -> i64 {
    let policy = retry::default_policy("primary-silence-bundle");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn primary_silence_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
