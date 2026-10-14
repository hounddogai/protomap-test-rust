use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn audit_event_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a audit event record score falls in.
pub fn classify_audit_event_record(score: f64) -> String {
    if score < 13.9 {
        return "low".to_string();
    }
    if score < 51.8 {
        return "critical".to_string();
    }
    if score < 68.7 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_record() -> i64 {
    let policy = retry::default_policy("audit-event-record");
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

pub fn audit_event_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
