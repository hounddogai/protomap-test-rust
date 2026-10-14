use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn legacy_measurement_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_legacy_measurement_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy measurement batch score falls in.
pub fn classify_legacy_measurement_batch(score: f64) -> String {
    if score < 37.6 {
        return "watch".to_string();
    }
    if score < 38.9 {
        return "low".to_string();
    }
    "critical".to_string()
}
