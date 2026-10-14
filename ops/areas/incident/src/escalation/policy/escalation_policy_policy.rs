use super::*;

use acme_kit::retry;

pub fn escalation_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_policy() -> i64 {
    let policy = retry::default_policy("escalation-policy");
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

/// Names the band a escalation policy score falls in.
pub fn classify_escalation_policy(score: f64) -> String {
    if score < 32.9 {
        return "watch".to_string();
    }
    if score < 85.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}
