use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a adaptive region pin rule score falls in.
pub fn classify_adaptive_region_pin_rule(score: f64) -> String {
    if score < 92.8 {
        return "watch".to_string();
    }
    if score < 98.0 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn adaptive_region_pin_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn merge_adaptive_region_pin_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn adaptive_region_pin_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
