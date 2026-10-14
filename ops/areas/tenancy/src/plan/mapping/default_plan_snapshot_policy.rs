use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_default_plan_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default plan snapshot code in its canonical DPS- form.
pub fn normalize_default_plan_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DPS-") {
        return clean.clone();
    }
    format!("DPS-{}", clean.replace(" ", "-"))
}

/// Names the band a default plan snapshot score falls in.
pub fn classify_default_plan_snapshot(score: f64) -> String {
    if score < 31.4 {
        return "watch".to_string();
    }
    if score < 86.9 {
        return "critical".to_string();
    }
    "high".to_string()
}
