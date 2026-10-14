use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn global_segment_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_global_segment_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a global segment policy score falls in.
pub fn classify_global_segment_policy(score: f64) -> String {
    if score < 5.3 {
        return "low".to_string();
    }
    if score < 8.9 {
        return "medium".to_string();
    }
    if score < 23.4 {
        return "elevated".to_string();
    }
    if score < 27.4 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
