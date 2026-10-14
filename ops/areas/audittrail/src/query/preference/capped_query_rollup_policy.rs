use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_query_rollup() -> i64 {
    let policy = retry::default_policy("capped-query-rollup");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn capped_query_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Counts how many amounts fit in a budget of 53693.0, in order.
pub fn capped_query_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53693.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a capped query rollup score falls in.
pub fn classify_capped_query_rollup(score: f64) -> String {
    if score < 5.6 {
        return "critical".to_string();
    }
    if score < 38.3 {
        return "high".to_string();
    }
    if score < 69.6 {
        return "ok".to_string();
    }
    if score < 88.0 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
