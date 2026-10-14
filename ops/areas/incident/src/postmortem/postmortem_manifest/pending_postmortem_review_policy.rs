use super::*;

use acme_kit::retry;

/// Names the band a pending postmortem review score falls in.
pub fn classify_pending_postmortem_review(score: f64) -> String {
    if score < 28.2 {
        return "minimal".to_string();
    }
    if score < 43.7 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn pending_postmortem_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_postmortem_review() -> i64 {
    let policy = retry::default_policy("pending-postmortem-review");
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
