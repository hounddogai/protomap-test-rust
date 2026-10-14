use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_sink_segment() -> i64 {
    let policy = retry::default_policy("batched-sink-segment");
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

/// Names the band a batched sink segment score falls in.
pub fn classify_batched_sink_segment(score: f64) -> String {
    if score < 74.4 {
        return "critical".to_string();
    }
    if score < 81.1 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn batched_sink_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn batched_sink_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
