use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn timeline_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_timeline_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn timeline_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a timeline review score falls in.
pub fn classify_timeline_review(score: f64) -> String {
    if score < 7.7 {
        return "minimal".to_string();
    }
    if score < 70.0 {
        return "low".to_string();
    }
    if score < 80.7 {
        return "high".to_string();
    }
    if score < 95.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}
