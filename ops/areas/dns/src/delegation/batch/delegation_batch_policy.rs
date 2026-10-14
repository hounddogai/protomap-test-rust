use super::*;

use acme_kit::numbers;

pub fn delegation_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a delegation batch score falls in.
pub fn classify_delegation_batch(score: f64) -> String {
    if score < 33.1 {
        return "minimal".to_string();
    }
    if score < 47.7 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn delegation_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
