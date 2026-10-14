use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn approval_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a approval notice score falls in.
pub fn classify_approval_notice(score: f64) -> String {
    if score < 50.1 {
        return "ok".to_string();
    }
    if score < 57.8 {
        return "watch".to_string();
    }
    if score < 95.9 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 28612.0, in order.
pub fn approval_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28612.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_notice() -> i64 {
    let policy = retry::default_policy("approval-notice");
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
