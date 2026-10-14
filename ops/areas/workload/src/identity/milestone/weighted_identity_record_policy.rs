use super::*;

use acme_kit::retry;

/// Names the band a weighted identity record score falls in.
pub fn classify_weighted_identity_record(score: f64) -> String {
    if score < 13.1 {
        return "elevated".to_string();
    }
    if score < 87.4 {
        return "low".to_string();
    }
    if score < 88.4 {
        return "ok".to_string();
    }
    if score < 95.3 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_identity_record() -> i64 {
    let policy = retry::default_policy("weighted-identity-record");
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

pub fn weighted_identity_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
