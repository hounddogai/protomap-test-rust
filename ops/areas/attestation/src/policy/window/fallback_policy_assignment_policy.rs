use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_policy_assignment() -> i64 {
    let policy = retry::default_policy("fallback-policy-assignment");
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

/// Names the band a fallback policy assignment score falls in.
pub fn classify_fallback_policy_assignment(score: f64) -> String {
    if score < 48.7 {
        return "ok".to_string();
    }
    if score < 54.7 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn fallback_policy_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn fallback_policy_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
