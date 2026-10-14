use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn primary_binding_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_primary_binding_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a primary binding rollup score falls in.
pub fn classify_primary_binding_rollup(score: f64) -> String {
    if score < 26.4 {
        return "critical".to_string();
    }
    if score < 51.8 {
        return "low".to_string();
    }
    if score < 65.0 {
        return "high".to_string();
    }
    "ok".to_string()
}
