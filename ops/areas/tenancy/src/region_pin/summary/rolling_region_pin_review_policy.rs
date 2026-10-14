use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a rolling region pin review score falls in.
pub fn classify_rolling_region_pin_review(score: f64) -> String {
    if score < 26.2 {
        return "watch".to_string();
    }
    if score < 27.0 {
        return "elevated".to_string();
    }
    if score < 28.0 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn rolling_region_pin_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn merge_rolling_region_pin_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
