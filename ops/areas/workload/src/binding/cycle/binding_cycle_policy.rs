use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn binding_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a binding cycle score falls in.
pub fn classify_binding_cycle(score: f64) -> String {
    if score < 17.7 {
        return "ok".to_string();
    }
    if score < 26.4 {
        return "minimal".to_string();
    }
    if score < 63.2 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn merge_binding_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn binding_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
