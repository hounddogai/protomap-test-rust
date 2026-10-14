use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_overage_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a overage manifest code in its canonical OM- form.
pub fn normalize_overage_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OM-") {
        return clean.clone();
    }
    format!("OM-{}", clean.replace(" ", "-"))
}
