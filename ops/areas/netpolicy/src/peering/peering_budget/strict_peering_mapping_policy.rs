use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_strict_peering_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a strict peering mapping score falls in.
pub fn classify_strict_peering_mapping(score: f64) -> String {
    if score < 6.2 {
        return "low".to_string();
    }
    if score < 37.8 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Puts a strict peering mapping code in its canonical SPM- form.
pub fn normalize_strict_peering_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPM-") {
        return clean.clone();
    }
    format!("SPM-{}", clean.replace(" ", "-"))
}
