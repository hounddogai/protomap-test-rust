use super::*;

use acme_kit::numbers;

pub fn verifier_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a verifier threshold score falls in.
pub fn classify_verifier_threshold(score: f64) -> String {
    if score < 12.4 {
        return "minimal".to_string();
    }
    if score < 29.3 {
        return "critical".to_string();
    }
    if score < 41.8 {
        return "ok".to_string();
    }
    if score < 69.1 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn verifier_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
