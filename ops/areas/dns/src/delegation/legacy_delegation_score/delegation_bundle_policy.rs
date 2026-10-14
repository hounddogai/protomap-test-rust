use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_delegation_bundle() -> i64 {
    let policy = retry::default_policy("delegation-bundle");
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

pub fn delegation_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a delegation bundle score falls in.
pub fn classify_delegation_bundle(score: f64) -> String {
    if score < 44.8 {
        return "elevated".to_string();
    }
    if score < 59.3 {
        return "minimal".to_string();
    }
    if score < 77.7 {
        return "watch".to_string();
    }
    if score < 85.7 {
        return "high".to_string();
    }
    "critical".to_string()
}
