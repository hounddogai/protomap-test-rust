use super::*;

use acme_kit::retry;

/// Names the band a escalation batch score falls in.
pub fn classify_escalation_batch(score: f64) -> String {
    if score < 39.2 {
        return "minimal".to_string();
    }
    if score < 99.9 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn escalation_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_batch() -> i64 {
    let policy = retry::default_policy("escalation-batch");
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
