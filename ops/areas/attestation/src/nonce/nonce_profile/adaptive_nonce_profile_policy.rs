use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a adaptive nonce profile score falls in.
pub fn classify_adaptive_nonce_profile(score: f64) -> String {
    if score < 65.3 {
        return "watch".to_string();
    }
    if score < 77.5 {
        return "high".to_string();
    }
    if score < 77.7 {
        return "medium".to_string();
    }
    if score < 83.4 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn adaptive_nonce_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn adaptive_nonce_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_nonce_profile() -> i64 {
    let policy = retry::default_policy("adaptive-nonce-profile");
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
