use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_strict_query_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn strict_query_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a strict query threshold code in its canonical SQT- form.
pub fn normalize_strict_query_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQT-") {
        return clean.clone();
    }
    format!("SQT-{}", clean.replace(" ", "-"))
}

/// Names the band a strict query threshold score falls in.
pub fn classify_strict_query_threshold(score: f64) -> String {
    if score < 10.3 {
        return "critical".to_string();
    }
    if score < 65.8 {
        return "low".to_string();
    }
    if score < 95.2 {
        return "high".to_string();
    }
    "medium".to_string()
}
