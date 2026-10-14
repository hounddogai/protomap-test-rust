use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn alert_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_request() -> i64 {
    let policy = retry::default_policy("alert-request");
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

/// Names the band a alert request score falls in.
pub fn classify_alert_request(score: f64) -> String {
    if score < 24.0 {
        return "watch".to_string();
    }
    if score < 58.9 {
        return "high".to_string();
    }
    if score < 67.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn alert_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
