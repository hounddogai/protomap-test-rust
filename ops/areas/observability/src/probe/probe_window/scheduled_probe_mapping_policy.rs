use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a scheduled probe mapping code in its canonical SPM- form.
pub fn normalize_scheduled_probe_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPM-") {
        return clean.clone();
    }
    format!("SPM-{}", clean.replace(" ", "-"))
}

pub fn merge_scheduled_probe_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled probe mapping score falls in.
pub fn classify_scheduled_probe_mapping(score: f64) -> String {
    if score < 5.5 {
        return "minimal".to_string();
    }
    if score < 49.8 {
        return "watch".to_string();
    }
    if score < 96.6 {
        return "low".to_string();
    }
    "critical".to_string()
}
