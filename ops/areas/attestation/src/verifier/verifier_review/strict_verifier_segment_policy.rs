use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_verifier_segment() -> i64 {
    let policy = retry::default_policy("strict-verifier-segment");
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

pub fn strict_verifier_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a strict verifier segment score falls in.
pub fn classify_strict_verifier_segment(score: f64) -> String {
    if score < 12.9 {
        return "ok".to_string();
    }
    if score < 37.9 {
        return "high".to_string();
    }
    if score < 71.5 {
        return "watch".to_string();
    }
    if score < 77.1 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 96226.0, in order.
pub fn strict_verifier_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96226.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
