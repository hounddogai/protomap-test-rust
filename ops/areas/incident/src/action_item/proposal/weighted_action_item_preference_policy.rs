use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a weighted action item preference score falls in.
pub fn classify_weighted_action_item_preference(score: f64) -> String {
    if score < 3.2 {
        return "minimal".to_string();
    }
    if score < 88.5 {
        return "critical".to_string();
    }
    if score < 90.7 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn merge_weighted_action_item_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
