use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a usage tier score falls in.
pub fn classify_usage_tier(score: f64) -> String {
    if score < 5.9 {
        return "critical".to_string();
    }
    if score < 14.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn merge_usage_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn usage_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
