use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn rollout_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_rollout_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rollout record score falls in.
pub fn classify_rollout_record(score: f64) -> String {
    if score < 17.9 {
        return "watch".to_string();
    }
    if score < 60.5 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
