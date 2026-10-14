use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn action_item_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a action item mapping score falls in.
pub fn classify_action_item_mapping(score: f64) -> String {
    if score < 4.8 {
        return "high".to_string();
    }
    if score < 18.9 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn action_item_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_action_item_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
