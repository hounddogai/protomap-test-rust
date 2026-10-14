use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_rule_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 79032.0, in order.
pub fn adaptive_rule_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79032.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_rule_assignment() -> i64 {
    let policy = retry::default_policy("adaptive-rule-assignment");
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

pub fn adaptive_rule_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.18;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
