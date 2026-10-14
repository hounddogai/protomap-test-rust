use super::*;

use acme_kit::retry;

pub fn region_pin_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 8649.0, in order.
pub fn region_pin_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8649.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a region pin notice score falls in.
pub fn classify_region_pin_notice(score: f64) -> String {
    if score < 52.3 {
        return "high".to_string();
    }
    if score < 56.3 {
        return "elevated".to_string();
    }
    if score < 84.5 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_notice() -> i64 {
    let policy = retry::default_policy("region-pin-notice");
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
