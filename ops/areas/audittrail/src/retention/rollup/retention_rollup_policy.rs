use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_retention_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a retention rollup score falls in.
pub fn classify_retention_rollup(score: f64) -> String {
    if score < 9.5 {
        return "low".to_string();
    }
    if score < 38.1 {
        return "elevated".to_string();
    }
    if score < 52.7 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn retention_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn retention_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
