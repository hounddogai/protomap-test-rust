use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a adaptive alert variance score falls in.
pub fn classify_adaptive_alert_variance(score: f64) -> String {
    if score < 18.5 {
        return "minimal".to_string();
    }
    if score < 51.9 {
        return "low".to_string();
    }
    if score < 74.2 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn adaptive_alert_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn merge_adaptive_alert_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn adaptive_alert_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
