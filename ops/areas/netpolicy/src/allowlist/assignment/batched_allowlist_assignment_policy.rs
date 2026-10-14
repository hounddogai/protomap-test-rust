use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn batched_allowlist_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_allowlist_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_batched_allowlist_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched allowlist assignment score falls in.
pub fn classify_batched_allowlist_assignment(score: f64) -> String {
    if score < 25.8 {
        return "low".to_string();
    }
    if score < 61.4 {
        return "medium".to_string();
    }
    if score < 63.6 {
        return "minimal".to_string();
    }
    if score < 68.6 {
        return "critical".to_string();
    }
    "ok".to_string()
}
