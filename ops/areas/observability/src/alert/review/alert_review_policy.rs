use super::*;

use acme_kit::retry;

/// Names the band a alert review score falls in.
pub fn classify_alert_review(score: f64) -> String {
    if score < 6.0 {
        return "medium".to_string();
    }
    if score < 11.8 {
        return "high".to_string();
    }
    if score < 88.9 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_review() -> i64 {
    let policy = retry::default_policy("alert-review");
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

pub fn alert_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
