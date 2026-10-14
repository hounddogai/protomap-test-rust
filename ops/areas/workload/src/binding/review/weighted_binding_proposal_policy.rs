use super::*;

use acme_kit::retry;

pub fn weighted_binding_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a weighted binding proposal score falls in.
pub fn classify_weighted_binding_proposal(score: f64) -> String {
    if score < 20.8 {
        return "elevated".to_string();
    }
    if score < 38.3 {
        return "high".to_string();
    }
    if score < 68.0 {
        return "critical".to_string();
    }
    if score < 92.0 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_binding_proposal() -> i64 {
    let policy = retry::default_policy("weighted-binding-proposal");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
