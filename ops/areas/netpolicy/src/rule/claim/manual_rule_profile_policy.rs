use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_rule_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 38726.0, in order.
pub fn manual_rule_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38726.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_rule_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_rule_profile() -> i64 {
    let policy = retry::default_policy("manual-rule-profile");
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
