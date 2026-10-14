use super::*;

use acme_kit::retry;

/// Names the band a evidence assignment score falls in.
pub fn classify_evidence_assignment(score: f64) -> String {
    if score < 25.4 {
        return "elevated".to_string();
    }
    if score < 38.8 {
        return "high".to_string();
    }
    if score < 41.4 {
        return "critical".to_string();
    }
    if score < 99.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_assignment() -> i64 {
    let policy = retry::default_policy("evidence-assignment");
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

pub fn evidence_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a evidence assignment code in its canonical EA- form.
pub fn normalize_evidence_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EA-") {
        return clean.clone();
    }
    format!("EA-{}", clean.replace(" ", "-"))
}
