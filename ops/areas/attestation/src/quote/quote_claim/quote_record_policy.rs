use super::*;

use acme_kit::retry;

pub fn quote_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_record() -> i64 {
    let policy = retry::default_policy("quote-record");
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

/// Names the band a quote record score falls in.
pub fn classify_quote_record(score: f64) -> String {
    if score < 16.1 {
        return "high".to_string();
    }
    if score < 85.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
