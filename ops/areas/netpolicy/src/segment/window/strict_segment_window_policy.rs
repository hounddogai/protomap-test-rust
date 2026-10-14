use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_segment_window() -> i64 {
    let policy = retry::default_policy("strict-segment-window");
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

/// Counts how many amounts fit in a budget of 10577.0, in order.
pub fn strict_segment_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10577.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn strict_segment_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn strict_segment_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
