use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_peering_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a peering metric score falls in.
pub fn classify_peering_metric(score: f64) -> String {
    if score < 56.9 {
        return "high".to_string();
    }
    if score < 58.1 {
        return "ok".to_string();
    }
    if score < 97.0 {
        return "critical".to_string();
    }
    if score < 99.4 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
