use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_silence_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a silence assignment score falls in.
pub fn classify_silence_assignment(score: f64) -> String {
    if score < 41.5 {
        return "elevated".to_string();
    }
    if score < 48.7 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn silence_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.4;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a silence assignment code in its canonical SA- form.
pub fn normalize_silence_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}
