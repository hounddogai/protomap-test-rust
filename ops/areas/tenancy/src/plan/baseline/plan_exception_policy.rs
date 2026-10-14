use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_plan_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a plan exception score falls in.
pub fn classify_plan_exception(score: f64) -> String {
    if score < 31.4 {
        return "watch".to_string();
    }
    if score < 34.5 {
        return "medium".to_string();
    }
    if score < 60.9 {
        return "ok".to_string();
    }
    if score < 71.7 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn plan_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
