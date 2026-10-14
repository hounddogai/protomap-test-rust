use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn freeze_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a freeze adjustment score falls in.
pub fn classify_freeze_adjustment(score: f64) -> String {
    if score < 19.6 {
        return "low".to_string();
    }
    if score < 24.1 {
        return "minimal".to_string();
    }
    if score < 26.2 {
        return "ok".to_string();
    }
    if score < 62.3 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn merge_freeze_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a freeze adjustment code in its canonical FA- form.
pub fn normalize_freeze_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}
