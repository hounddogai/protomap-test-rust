use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_region_pin_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback region pin mapping code in its canonical FRPM- form.
pub fn normalize_fallback_region_pin_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRPM-") {
        return clean.clone();
    }
    format!("FRPM-{}", clean.replace(" ", "-"))
}

pub fn merge_fallback_region_pin_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
