use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_segment_adjustment() -> i64 {
    let policy = retry::default_policy("tiered-segment-adjustment");
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

/// Names the band a tiered segment adjustment score falls in.
pub fn classify_tiered_segment_adjustment(score: f64) -> String {
    if score < 49.9 {
        return "critical".to_string();
    }
    if score < 79.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn tiered_segment_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 85533.0, in order.
pub fn tiered_segment_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85533.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
