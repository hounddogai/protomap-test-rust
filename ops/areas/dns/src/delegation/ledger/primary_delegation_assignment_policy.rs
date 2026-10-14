use super::*;

use acme_kit::retry;

/// Names the band a primary delegation assignment score falls in.
pub fn classify_primary_delegation_assignment(score: f64) -> String {
    if score < 10.5 {
        return "medium".to_string();
    }
    if score < 36.8 {
        return "watch".to_string();
    }
    if score < 43.6 {
        return "critical".to_string();
    }
    if score < 67.4 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn primary_delegation_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_delegation_assignment() -> i64 {
    let policy = retry::default_policy("primary-delegation-assignment");
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
