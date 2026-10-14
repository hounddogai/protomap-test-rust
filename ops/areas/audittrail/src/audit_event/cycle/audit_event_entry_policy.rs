use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_entry() -> i64 {
    let policy = retry::default_policy("audit-event-entry");
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

/// Names the band a audit event entry score falls in.
pub fn classify_audit_event_entry(score: f64) -> String {
    if score < 51.3 {
        return "elevated".to_string();
    }
    if score < 53.3 {
        return "low".to_string();
    }
    if score < 77.4 {
        return "medium".to_string();
    }
    if score < 92.5 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn audit_event_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
