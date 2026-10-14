use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_federation_bundle() -> i64 {
    let policy = retry::default_policy("primary-federation-bundle");
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

pub fn primary_federation_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a primary federation bundle score falls in.
pub fn classify_primary_federation_bundle(score: f64) -> String {
    if score < 52.1 {
        return "high".to_string();
    }
    if score < 86.1 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
