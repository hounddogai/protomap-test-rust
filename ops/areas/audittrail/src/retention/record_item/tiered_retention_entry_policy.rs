use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_retention_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered retention entry code in its canonical TRE- form.
pub fn normalize_tiered_retention_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRE-") {
        return clean.clone();
    }
    format!("TRE-{}", clean.replace(" ", "-"))
}
