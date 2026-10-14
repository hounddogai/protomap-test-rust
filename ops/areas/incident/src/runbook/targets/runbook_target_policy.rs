use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_runbook_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn runbook_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a runbook target score falls in.
pub fn classify_runbook_target(score: f64) -> String {
    if score < 19.6 {
        return "elevated".to_string();
    }
    if score < 69.7 {
        return "high".to_string();
    }
    "ok".to_string()
}
