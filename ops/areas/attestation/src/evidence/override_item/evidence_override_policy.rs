use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_evidence_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a evidence override code in its canonical EO- form.
pub fn normalize_evidence_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EO-") {
        return clean.clone();
    }
    format!("EO-{}", clean.replace(" ", "-"))
}
