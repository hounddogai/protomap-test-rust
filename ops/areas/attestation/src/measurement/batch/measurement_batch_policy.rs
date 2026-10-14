use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn measurement_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a measurement batch score falls in.
pub fn classify_measurement_batch(score: f64) -> String {
    if score < 2.2 {
        return "ok".to_string();
    }
    if score < 12.6 {
        return "low".to_string();
    }
    if score < 80.2 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_measurement_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
