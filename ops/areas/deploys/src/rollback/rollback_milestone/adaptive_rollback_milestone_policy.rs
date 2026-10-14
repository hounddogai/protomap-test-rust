use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn adaptive_rollback_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive rollback milestone score falls in.
pub fn classify_adaptive_rollback_milestone(score: f64) -> String {
    if score < 4.9 {
        return "elevated".to_string();
    }
    if score < 82.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn merge_adaptive_rollback_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
