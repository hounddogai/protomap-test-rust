use super::*;

use acme_kit::numbers;

/// Names the band a isolation exception score falls in.
pub fn classify_isolation_exception(score: f64) -> String {
    if score < 6.8 {
        return "watch".to_string();
    }
    if score < 9.8 {
        return "medium".to_string();
    }
    if score < 16.0 {
        return "critical".to_string();
    }
    if score < 55.2 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn isolation_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn isolation_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
