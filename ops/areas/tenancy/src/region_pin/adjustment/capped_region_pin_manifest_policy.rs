use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_capped_region_pin_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped region pin manifest score falls in.
pub fn classify_capped_region_pin_manifest(score: f64) -> String {
    if score < 16.1 {
        return "minimal".to_string();
    }
    if score < 20.7 {
        return "low".to_string();
    }
    if score < 46.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a capped region pin manifest code in its canonical CRPM- form.
pub fn normalize_capped_region_pin_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRPM-") {
        return clean.clone();
    }
    format!("CRPM-{}", clean.replace(" ", "-"))
}
