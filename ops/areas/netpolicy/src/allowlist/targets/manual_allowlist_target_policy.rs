use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a manual allowlist target score falls in.
pub fn classify_manual_allowlist_target(score: f64) -> String {
    if score < 64.3 {
        return "elevated".to_string();
    }
    if score < 68.2 {
        return "medium".to_string();
    }
    if score < 85.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn merge_manual_allowlist_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_allowlist_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
