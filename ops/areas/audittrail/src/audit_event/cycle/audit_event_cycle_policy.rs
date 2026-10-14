use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_audit_event_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn audit_event_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a audit event cycle score falls in.
pub fn classify_audit_event_cycle(score: f64) -> String {
    if score < 65.2 {
        return "elevated".to_string();
    }
    if score < 95.0 {
        return "low".to_string();
    }
    "minimal".to_string()
}
