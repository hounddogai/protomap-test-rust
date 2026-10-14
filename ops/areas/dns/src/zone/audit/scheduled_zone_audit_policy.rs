use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn scheduled_zone_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn merge_scheduled_zone_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled zone audit score falls in.
pub fn classify_scheduled_zone_audit(score: f64) -> String {
    if score < 26.5 {
        return "elevated".to_string();
    }
    if score < 52.8 {
        return "minimal".to_string();
    }
    if score < 57.4 {
        return "high".to_string();
    }
    "medium".to_string()
}
