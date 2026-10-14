use super::*;

use acme_kit::numbers;

pub fn isolation_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

pub fn isolation_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a isolation bundle score falls in.
pub fn classify_isolation_bundle(score: f64) -> String {
    if score < 48.8 {
        return "low".to_string();
    }
    if score < 94.2 {
        return "watch".to_string();
    }
    "medium".to_string()
}
