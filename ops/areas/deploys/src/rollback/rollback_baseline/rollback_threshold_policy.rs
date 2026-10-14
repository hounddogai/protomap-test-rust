use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn rollback_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_rollback_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rollback threshold score falls in.
pub fn classify_rollback_threshold(score: f64) -> String {
    if score < 13.9 {
        return "high".to_string();
    }
    if score < 55.9 {
        return "ok".to_string();
    }
    if score < 87.6 {
        return "low".to_string();
    }
    "watch".to_string()
}
