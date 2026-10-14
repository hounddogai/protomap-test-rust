use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a usage segment code in its canonical US- form.
pub fn normalize_usage_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("US-") {
        return clean.clone();
    }
    format!("US-{}", clean.replace(" ", "-"))
}

pub fn usage_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a usage segment score falls in.
pub fn classify_usage_segment(score: f64) -> String {
    if score < 35.3 {
        return "watch".to_string();
    }
    if score < 82.9 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn merge_usage_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
