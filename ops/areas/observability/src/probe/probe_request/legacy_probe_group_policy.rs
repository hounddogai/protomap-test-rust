use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn legacy_probe_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a legacy probe group score falls in.
pub fn classify_legacy_probe_group(score: f64) -> String {
    if score < 4.9 {
        return "high".to_string();
    }
    if score < 11.8 {
        return "medium".to_string();
    }
    if score < 71.1 {
        return "elevated".to_string();
    }
    if score < 90.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn legacy_probe_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_legacy_probe_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
