use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a lenient token reservation score falls in.
pub fn classify_lenient_token_reservation(score: f64) -> String {
    if score < 45.3 {
        return "critical".to_string();
    }
    if score < 60.7 {
        return "high".to_string();
    }
    if score < 65.7 {
        return "watch".to_string();
    }
    if score < 82.2 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_token_reservation() -> i64 {
    let policy = retry::default_policy("lenient-token-reservation");
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

pub fn lenient_token_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn lenient_token_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
