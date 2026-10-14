use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_action_item_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn action_item_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a action item budget score falls in.
pub fn classify_action_item_budget(score: f64) -> String {
    if score < 10.2 {
        return "critical".to_string();
    }
    if score < 46.5 {
        return "minimal".to_string();
    }
    if score < 51.7 {
        return "elevated".to_string();
    }
    if score < 99.8 {
        return "watch".to_string();
    }
    "ok".to_string()
}
