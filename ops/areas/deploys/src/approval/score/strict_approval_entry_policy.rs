use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a strict approval entry score falls in.
pub fn classify_strict_approval_entry(score: f64) -> String {
    if score < 8.1 {
        return "high".to_string();
    }
    if score < 44.8 {
        return "low".to_string();
    }
    if score < 83.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn strict_approval_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_strict_approval_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
