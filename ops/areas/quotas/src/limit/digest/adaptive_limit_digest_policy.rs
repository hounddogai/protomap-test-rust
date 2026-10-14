use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_limit_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a adaptive limit digest score falls in.
pub fn classify_adaptive_limit_digest(score: f64) -> String {
    if score < 1.5 {
        return "low".to_string();
    }
    if score < 1.7 {
        return "critical".to_string();
    }
    if score < 91.7 {
        return "high".to_string();
    }
    if score < 93.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_limit_digest() -> i64 {
    let policy = retry::default_policy("adaptive-limit-digest");
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

pub fn adaptive_limit_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
