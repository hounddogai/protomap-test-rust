use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_action_item_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a action item adjustment score falls in.
pub fn classify_action_item_adjustment(score: f64) -> String {
    if score < 47.9 {
        return "low".to_string();
    }
    if score < 72.8 {
        return "medium".to_string();
    }
    "high".to_string()
}
