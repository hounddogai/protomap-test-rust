use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_peering_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a peering allocation score falls in.
pub fn classify_peering_allocation(score: f64) -> String {
    if score < 18.1 {
        return "watch".to_string();
    }
    if score < 19.9 {
        return "medium".to_string();
    }
    if score < 54.0 {
        return "critical".to_string();
    }
    if score < 99.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}
