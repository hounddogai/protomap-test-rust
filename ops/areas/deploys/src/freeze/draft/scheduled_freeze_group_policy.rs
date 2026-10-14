use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn scheduled_freeze_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_scheduled_freeze_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled freeze group score falls in.
pub fn classify_scheduled_freeze_group(score: f64) -> String {
    if score < 9.2 {
        return "medium".to_string();
    }
    if score < 36.7 {
        return "low".to_string();
    }
    "elevated".to_string()
}
