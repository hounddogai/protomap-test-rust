use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a batched zone plan score falls in.
pub fn classify_batched_zone_plan(score: f64) -> String {
    if score < 65.6 {
        return "critical".to_string();
    }
    if score < 76.2 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn batched_zone_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_batched_zone_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
