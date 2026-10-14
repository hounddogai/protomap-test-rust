use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_peering_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a peering mapping score falls in.
pub fn classify_peering_mapping(score: f64) -> String {
    if score < 34.4 {
        return "low".to_string();
    }
    if score < 57.2 {
        return "elevated".to_string();
    }
    if score < 60.4 {
        return "ok".to_string();
    }
    if score < 73.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn peering_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a peering mapping code in its canonical PM- form.
pub fn normalize_peering_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}
