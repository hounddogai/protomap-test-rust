use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn scheduled_verifier_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_scheduled_verifier_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled verifier review score falls in.
pub fn classify_scheduled_verifier_review(score: f64) -> String {
    if score < 17.8 {
        return "medium".to_string();
    }
    if score < 54.1 {
        return "high".to_string();
    }
    if score < 76.3 {
        return "low".to_string();
    }
    if score < 85.7 {
        return "ok".to_string();
    }
    "critical".to_string()
}
