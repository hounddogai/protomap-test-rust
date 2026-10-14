use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_seasonal_credential_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn seasonal_credential_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn seasonal_credential_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a seasonal credential summary score falls in.
pub fn classify_seasonal_credential_summary(score: f64) -> String {
    if score < 35.5 {
        return "critical".to_string();
    }
    if score < 76.0 {
        return "high".to_string();
    }
    if score < 89.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}
