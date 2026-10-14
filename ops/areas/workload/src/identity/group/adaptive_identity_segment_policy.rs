use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_adaptive_identity_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a adaptive identity segment code in its canonical AIS- form.
pub fn normalize_adaptive_identity_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIS-") {
        return clean.clone();
    }
    format!("AIS-{}", clean.replace(" ", "-"))
}
