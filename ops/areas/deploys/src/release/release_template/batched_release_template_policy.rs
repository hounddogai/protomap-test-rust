use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a batched release template score falls in.
pub fn classify_batched_release_template(score: f64) -> String {
    if score < 15.6 {
        return "ok".to_string();
    }
    if score < 84.5 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Puts a batched release template code in its canonical BRT- form.
pub fn normalize_batched_release_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRT-") {
        return clean.clone();
    }
    format!("BRT-{}", clean.replace(" ", "-"))
}

pub fn batched_release_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_batched_release_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
