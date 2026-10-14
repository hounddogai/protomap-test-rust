use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary runbook mapping score falls in.
pub fn classify_primary_runbook_mapping(score: f64) -> String {
    if score < 32.6 {
        return "high".to_string();
    }
    if score < 43.6 {
        return "elevated".to_string();
    }
    if score < 79.1 {
        return "medium".to_string();
    }
    if score < 94.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn merge_primary_runbook_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
