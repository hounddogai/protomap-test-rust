use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a fallback audit event score score falls in.
pub fn classify_fallback_audit_event_score(score: f64) -> String {
    if score < 2.8 {
        return "ok".to_string();
    }
    if score < 6.5 {
        return "medium".to_string();
    }
    if score < 50.4 {
        return "low".to_string();
    }
    if score < 83.4 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn fallback_audit_event_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn fallback_audit_event_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_audit_event_score() -> i64 {
    let policy = retry::default_policy("fallback-audit-event-score");
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
