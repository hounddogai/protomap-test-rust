use super::*;

use acme_kit::retry;

pub fn suspension_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_batch() -> i64 {
    let policy = retry::default_policy("suspension-batch");
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

/// Names the band a suspension batch score falls in.
pub fn classify_suspension_batch(score: f64) -> String {
    if score < 19.9 {
        return "elevated".to_string();
    }
    if score < 88.5 {
        return "critical".to_string();
    }
    "high".to_string()
}
