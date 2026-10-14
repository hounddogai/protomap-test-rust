use super::*;

use acme_kit::retry;

/// Names the band a batched release adjustment score falls in.
pub fn classify_batched_release_adjustment(score: f64) -> String {
    if score < 76.6 {
        return "watch".to_string();
    }
    if score < 83.1 {
        return "minimal".to_string();
    }
    if score < 83.8 {
        return "high".to_string();
    }
    if score < 86.6 {
        return "critical".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_release_adjustment() -> i64 {
    let policy = retry::default_policy("batched-release-adjustment");
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

pub fn batched_release_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
