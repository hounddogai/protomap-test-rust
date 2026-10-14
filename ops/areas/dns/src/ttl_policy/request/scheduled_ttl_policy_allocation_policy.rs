use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a scheduled ttl policy allocation score falls in.
pub fn classify_scheduled_ttl_policy_allocation(score: f64) -> String {
    if score < 47.0 {
        return "minimal".to_string();
    }
    if score < 59.4 {
        return "watch".to_string();
    }
    if score < 88.4 {
        return "elevated".to_string();
    }
    if score < 92.3 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn scheduled_ttl_policy_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_ttl_policy_allocation() -> i64 {
    let policy = retry::default_policy("scheduled-ttl-policy-allocation");
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
