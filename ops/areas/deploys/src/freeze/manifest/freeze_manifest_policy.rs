use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_freeze_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a freeze manifest score falls in.
pub fn classify_freeze_manifest(score: f64) -> String {
    if score < 16.1 {
        return "high".to_string();
    }
    if score < 94.4 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn freeze_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
