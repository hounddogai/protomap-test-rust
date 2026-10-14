use super::*;

use acme_kit::retry;

pub fn batched_nonce_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a batched nonce review code in its canonical BNR- form.
pub fn normalize_batched_nonce_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BNR-") {
        return clean.clone();
    }
    format!("BNR-{}", clean.replace(" ", "-"))
}

/// Names the band a batched nonce review score falls in.
pub fn classify_batched_nonce_review(score: f64) -> String {
    if score < 17.1 {
        return "watch".to_string();
    }
    if score < 49.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_nonce_review() -> i64 {
    let policy = retry::default_policy("batched-nonce-review");
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
