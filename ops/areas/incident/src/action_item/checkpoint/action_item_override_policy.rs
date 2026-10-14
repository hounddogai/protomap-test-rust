use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_action_item_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a action item override score falls in.
pub fn classify_action_item_override(score: f64) -> String {
    if score < 46.8 {
        return "watch".to_string();
    }
    if score < 53.7 {
        return "minimal".to_string();
    }
    if score < 54.6 {
        return "low".to_string();
    }
    if score < 68.8 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn action_item_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.18;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
