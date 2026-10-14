use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn batched_verifier_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn batched_verifier_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched verifier bundle score falls in.
pub fn classify_batched_verifier_bundle(score: f64) -> String {
    if score < 30.6 {
        return "minimal".to_string();
    }
    if score < 82.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_verifier_bundle() -> i64 {
    let policy = retry::default_policy("batched-verifier-bundle");
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
