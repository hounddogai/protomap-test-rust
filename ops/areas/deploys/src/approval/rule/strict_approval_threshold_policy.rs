use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn strict_approval_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn strict_approval_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a strict approval threshold score falls in.
pub fn classify_strict_approval_threshold(score: f64) -> String {
    if score < 20.1 {
        return "low".to_string();
    }
    if score < 38.6 {
        return "minimal".to_string();
    }
    if score < 81.6 {
        return "critical".to_string();
    }
    if score < 91.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn merge_strict_approval_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
