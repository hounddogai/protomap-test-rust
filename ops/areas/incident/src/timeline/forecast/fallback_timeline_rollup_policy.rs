use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_timeline_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback timeline rollup score falls in.
pub fn classify_fallback_timeline_rollup(score: f64) -> String {
    if score < 24.6 {
        return "elevated".to_string();
    }
    if score < 54.8 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn merge_fallback_timeline_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
