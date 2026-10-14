use super::*;

use acme_kit::numbers;

pub fn batched_failover_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a batched failover review score falls in.
pub fn classify_batched_failover_review(score: f64) -> String {
    if score < 13.9 {
        return "minimal".to_string();
    }
    if score < 41.2 {
        return "low".to_string();
    }
    if score < 47.9 {
        return "watch".to_string();
    }
    if score < 72.0 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn batched_failover_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a batched failover review code in its canonical BFR- form.
pub fn normalize_batched_failover_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BFR-") {
        return clean.clone();
    }
    format!("BFR-{}", clean.replace(" ", "-"))
}
