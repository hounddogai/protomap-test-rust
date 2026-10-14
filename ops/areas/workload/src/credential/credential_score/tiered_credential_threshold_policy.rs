use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a tiered credential threshold score falls in.
pub fn classify_tiered_credential_threshold(score: f64) -> String {
    if score < 10.9 {
        return "medium".to_string();
    }
    if score < 33.4 {
        return "critical".to_string();
    }
    if score < 72.2 {
        return "ok".to_string();
    }
    if score < 94.5 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_tiered_credential_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
