use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn identity_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn identity_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a identity preference score falls in.
pub fn classify_identity_preference(score: f64) -> String {
    if score < 13.1 {
        return "ok".to_string();
    }
    if score < 69.2 {
        return "elevated".to_string();
    }
    if score < 82.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_preference() -> i64 {
    let policy = retry::default_policy("identity-preference");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
