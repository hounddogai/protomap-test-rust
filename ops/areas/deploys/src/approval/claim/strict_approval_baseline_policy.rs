use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_approval_baseline() -> i64 {
    let policy = retry::default_policy("strict-approval-baseline");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a strict approval baseline score falls in.
pub fn classify_strict_approval_baseline(score: f64) -> String {
    if score < 4.2 {
        return "watch".to_string();
    }
    if score < 47.6 {
        return "minimal".to_string();
    }
    if score < 68.3 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn strict_approval_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn strict_approval_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
