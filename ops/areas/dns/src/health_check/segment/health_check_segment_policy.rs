use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_health_check_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a health check segment code in its canonical HCS- form.
pub fn normalize_health_check_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCS-") {
        return clean.clone();
    }
    format!("HCS-{}", clean.replace(" ", "-"))
}
