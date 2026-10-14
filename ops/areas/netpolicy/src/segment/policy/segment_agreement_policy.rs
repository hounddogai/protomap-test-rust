use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a segment agreement code in its canonical SA- form.
pub fn normalize_segment_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}

pub fn merge_segment_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn segment_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
