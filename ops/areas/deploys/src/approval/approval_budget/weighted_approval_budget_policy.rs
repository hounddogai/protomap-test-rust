use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn weighted_approval_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a weighted approval budget score falls in.
pub fn classify_weighted_approval_budget(score: f64) -> String {
    if score < 13.0 {
        return "minimal".to_string();
    }
    if score < 60.1 {
        return "low".to_string();
    }
    if score < 68.8 {
        return "ok".to_string();
    }
    if score < 68.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn merge_weighted_approval_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
