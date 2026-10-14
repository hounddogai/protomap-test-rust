use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_request() -> i64 {
    let policy = retry::default_policy("audit-event-request");
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

/// Names the band a audit event request score falls in.
pub fn classify_audit_event_request(score: f64) -> String {
    if score < 20.3 {
        return "critical".to_string();
    }
    if score < 32.3 {
        return "high".to_string();
    }
    if score < 59.8 {
        return "low".to_string();
    }
    if score < 84.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
