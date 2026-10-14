use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a peering review score falls in.
pub fn classify_peering_review(score: f64) -> String {
    if score < 11.5 {
        return "ok".to_string();
    }
    if score < 88.4 {
        return "minimal".to_string();
    }
    if score < 95.1 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn peering_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_peering_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
