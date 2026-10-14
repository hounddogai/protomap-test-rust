use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_evidence_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_evidence_reservation() -> i64 {
    let policy = retry::default_policy("fallback-evidence-reservation");
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

/// Names the band a fallback evidence reservation score falls in.
pub fn classify_fallback_evidence_reservation(score: f64) -> String {
    if score < 36.1 {
        return "minimal".to_string();
    }
    if score < 40.3 {
        return "ok".to_string();
    }
    if score < 61.7 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn fallback_evidence_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
