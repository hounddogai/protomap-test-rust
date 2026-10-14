use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_scheduled_peering_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled peering draft score falls in.
pub fn classify_scheduled_peering_draft(score: f64) -> String {
    if score < 15.0 {
        return "elevated".to_string();
    }
    if score < 74.7 {
        return "critical".to_string();
    }
    if score < 75.9 {
        return "low".to_string();
    }
    "ok".to_string()
}
