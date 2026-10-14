use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_zone_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a fallback zone slot code in its canonical FZS- form.
pub fn normalize_fallback_zone_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FZS-") {
        return clean.clone();
    }
    format!("FZS-{}", clean.replace(" ", "-"))
}

pub fn merge_fallback_zone_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn fallback_zone_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
