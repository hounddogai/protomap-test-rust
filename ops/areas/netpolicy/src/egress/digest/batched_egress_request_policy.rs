use super::*;

use acme_kit::retry;

/// Puts a batched egress request code in its canonical BER- form.
pub fn normalize_batched_egress_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BER-") {
        return clean.clone();
    }
    format!("BER-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_egress_request() -> i64 {
    let policy = retry::default_policy("batched-egress-request");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a batched egress request score falls in.
pub fn classify_batched_egress_request(score: f64) -> String {
    if score < 3.9 {
        return "high".to_string();
    }
    if score < 51.5 {
        return "low".to_string();
    }
    if score < 99.0 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn batched_egress_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
