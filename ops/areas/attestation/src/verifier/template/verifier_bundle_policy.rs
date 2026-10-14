use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a verifier bundle score falls in.
pub fn classify_verifier_bundle(score: f64) -> String {
    if score < 14.4 {
        return "high".to_string();
    }
    if score < 47.0 {
        return "watch".to_string();
    }
    if score < 88.7 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn verifier_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_verifier_bundle() -> i64 {
    let policy = retry::default_policy("verifier-bundle");
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
