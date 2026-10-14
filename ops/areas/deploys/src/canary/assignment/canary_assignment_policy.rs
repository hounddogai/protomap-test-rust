use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_canary_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn canary_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a canary assignment score falls in.
pub fn classify_canary_assignment(score: f64) -> String {
    if score < 37.7 {
        return "medium".to_string();
    }
    if score < 68.6 {
        return "high".to_string();
    }
    if score < 80.1 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
