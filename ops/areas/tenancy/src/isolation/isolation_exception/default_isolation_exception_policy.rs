use super::*;

use acme_kit::numbers;

/// Names the band a default isolation exception score falls in.
pub fn classify_default_isolation_exception(score: f64) -> String {
    if score < 11.9 {
        return "low".to_string();
    }
    if score < 20.8 {
        return "ok".to_string();
    }
    if score < 25.6 {
        return "critical".to_string();
    }
    if score < 42.0 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn default_isolation_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn default_isolation_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
