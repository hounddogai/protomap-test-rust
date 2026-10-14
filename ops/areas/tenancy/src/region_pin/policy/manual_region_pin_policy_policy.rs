use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn manual_region_pin_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_manual_region_pin_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a manual region pin policy code in its canonical MRPP- form.
pub fn normalize_manual_region_pin_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRPP-") {
        return clean.clone();
    }
    format!("MRPP-{}", clean.replace(" ", "-"))
}

pub fn manual_region_pin_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
