use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn global_release_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_global_release_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a global release estimate score falls in.
pub fn classify_global_release_estimate(score: f64) -> String {
    if score < 52.6 {
        return "low".to_string();
    }
    if score < 82.5 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
