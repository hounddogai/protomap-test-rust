use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn primary_freeze_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn merge_primary_freeze_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a primary freeze schedule score falls in.
pub fn classify_primary_freeze_schedule(score: f64) -> String {
    if score < 74.2 {
        return "watch".to_string();
    }
    if score < 79.7 {
        return "high".to_string();
    }
    "medium".to_string()
}
