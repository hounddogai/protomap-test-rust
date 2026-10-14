use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn default_release_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a default release snapshot score falls in.
pub fn classify_default_release_snapshot(score: f64) -> String {
    if score < 31.9 {
        return "elevated".to_string();
    }
    if score < 53.1 {
        return "low".to_string();
    }
    if score < 64.8 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_default_release_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default release snapshot code in its canonical DRS- form.
pub fn normalize_default_release_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRS-") {
        return clean.clone();
    }
    format!("DRS-{}", clean.replace(" ", "-"))
}
