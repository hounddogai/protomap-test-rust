use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a scheduled usage entry score falls in.
pub fn classify_scheduled_usage_entry(score: f64) -> String {
    if score < 14.2 {
        return "elevated".to_string();
    }
    if score < 23.1 {
        return "critical".to_string();
    }
    if score < 34.0 {
        return "low".to_string();
    }
    if score < 62.3 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn merge_scheduled_usage_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
