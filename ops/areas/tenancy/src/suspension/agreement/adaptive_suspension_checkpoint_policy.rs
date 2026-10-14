use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 67739.0, in order.
pub fn adaptive_suspension_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67739.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_suspension_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_suspension_checkpoint() -> i64 {
    let policy = retry::default_policy("adaptive-suspension-checkpoint");
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

pub fn adaptive_suspension_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
