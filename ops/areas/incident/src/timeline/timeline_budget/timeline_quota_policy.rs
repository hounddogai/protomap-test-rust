use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn timeline_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_quota() -> i64 {
    let policy = retry::default_policy("timeline-quota");
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

/// Names the band a timeline quota score falls in.
pub fn classify_timeline_quota(score: f64) -> String {
    if score < 19.8 {
        return "minimal".to_string();
    }
    if score < 51.6 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn timeline_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
