use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a segment preference code in its canonical SP- form.
pub fn normalize_segment_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SP-") {
        return clean.clone();
    }
    format!("SP-{}", clean.replace(" ", "-"))
}

pub fn merge_segment_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn segment_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a segment preference score falls in.
pub fn classify_segment_preference(score: f64) -> String {
    if score < 35.1 {
        return "ok".to_string();
    }
    if score < 37.5 {
        return "medium".to_string();
    }
    if score < 57.6 {
        return "watch".to_string();
    }
    if score < 89.0 {
        return "low".to_string();
    }
    "critical".to_string()
}
