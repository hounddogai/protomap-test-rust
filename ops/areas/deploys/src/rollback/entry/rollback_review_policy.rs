use super::*;

use acme_kit::retry;

pub fn rollback_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a rollback review code in its canonical RR- form.
pub fn normalize_rollback_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}

/// Names the band a rollback review score falls in.
pub fn classify_rollback_review(score: f64) -> String {
    if score < 4.7 {
        return "medium".to_string();
    }
    if score < 69.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_review() -> i64 {
    let policy = retry::default_policy("rollback-review");
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
