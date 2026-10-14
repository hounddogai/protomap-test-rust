use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_evidence_assignment() -> i64 {
    let policy = retry::default_policy("legacy-evidence-assignment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a legacy evidence assignment score falls in.
pub fn classify_legacy_evidence_assignment(score: f64) -> String {
    if score < 21.1 {
        return "critical".to_string();
    }
    if score < 25.6 {
        return "high".to_string();
    }
    if score < 49.0 {
        return "ok".to_string();
    }
    if score < 82.7 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn legacy_evidence_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
