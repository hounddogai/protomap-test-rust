use super::*;

use acme_kit::retry;

pub fn lenient_federation_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a lenient federation agreement score falls in.
pub fn classify_lenient_federation_agreement(score: f64) -> String {
    if score < 19.7 {
        return "watch".to_string();
    }
    if score < 34.7 {
        return "elevated".to_string();
    }
    if score < 69.1 {
        return "medium".to_string();
    }
    if score < 92.2 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_federation_agreement() -> i64 {
    let policy = retry::default_policy("lenient-federation-agreement");
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
