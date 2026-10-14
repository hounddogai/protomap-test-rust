use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_global_plan_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a global plan window score falls in.
pub fn classify_global_plan_window(score: f64) -> String {
    if score < 20.2 {
        return "low".to_string();
    }
    if score < 64.7 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn global_plan_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
