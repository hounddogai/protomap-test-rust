use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a region pin group code in its canonical RPG- form.
pub fn normalize_region_pin_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPG-") {
        return clean.clone();
    }
    format!("RPG-{}", clean.replace(" ", "-"))
}

pub fn region_pin_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a region pin group score falls in.
pub fn classify_region_pin_group(score: f64) -> String {
    if score < 31.1 {
        return "high".to_string();
    }
    if score < 49.3 {
        return "medium".to_string();
    }
    if score < 50.0 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn merge_region_pin_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
