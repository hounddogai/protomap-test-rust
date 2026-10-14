use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn policy_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_window() -> i64 {
    let policy = retry::default_policy("policy-window");
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

/// Names the band a policy window score falls in.
pub fn classify_policy_window(score: f64) -> String {
    if score < 10.2 {
        return "ok".to_string();
    }
    if score < 53.9 {
        return "low".to_string();
    }
    if score < 73.8 {
        return "medium".to_string();
    }
    if score < 76.7 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
