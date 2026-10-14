use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 25277.0, in order.
pub fn lenient_probe_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25277.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_probe_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_probe_milestone() -> i64 {
    let policy = retry::default_policy("lenient-probe-milestone");
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
