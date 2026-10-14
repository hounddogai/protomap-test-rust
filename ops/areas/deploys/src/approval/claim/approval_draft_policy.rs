use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a approval draft score falls in.
pub fn classify_approval_draft(score: f64) -> String {
    if score < 7.3 {
        return "watch".to_string();
    }
    if score < 20.4 {
        return "critical".to_string();
    }
    if score < 81.7 {
        return "ok".to_string();
    }
    if score < 91.9 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn approval_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_draft() -> i64 {
    let policy = retry::default_policy("approval-draft");
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

/// Counts how many amounts fit in a budget of 50018.0, in order.
pub fn approval_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50018.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
