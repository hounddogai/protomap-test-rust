use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn global_egress_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_egress_notice() -> i64 {
    let policy = retry::default_policy("global-egress-notice");
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

pub fn global_egress_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 28866.0, in order.
pub fn global_egress_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28866.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
