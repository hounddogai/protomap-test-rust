use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_suspension_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_suspension_adjustment() -> i64 {
    let policy = retry::default_policy("lenient-suspension-adjustment");
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

/// Names the band a lenient suspension adjustment score falls in.
pub fn classify_lenient_suspension_adjustment(score: f64) -> String {
    if score < 8.1 {
        return "critical".to_string();
    }
    if score < 45.9 {
        return "watch".to_string();
    }
    if score < 49.7 {
        return "minimal".to_string();
    }
    if score < 63.8 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn lenient_suspension_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
