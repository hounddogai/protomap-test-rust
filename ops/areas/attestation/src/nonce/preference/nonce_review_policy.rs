use super::*;

use acme_kit::numbers;

pub fn nonce_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn nonce_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a nonce review score falls in.
pub fn classify_nonce_review(score: f64) -> String {
    if score < 27.3 {
        return "low".to_string();
    }
    if score < 35.0 {
        return "watch".to_string();
    }
    if score < 78.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn nonce_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
