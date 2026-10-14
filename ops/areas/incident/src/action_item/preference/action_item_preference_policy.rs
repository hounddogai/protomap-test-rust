use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_action_item_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a action item preference score falls in.
pub fn classify_action_item_preference(score: f64) -> String {
    if score < 23.6 {
        return "elevated".to_string();
    }
    if score < 73.8 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
