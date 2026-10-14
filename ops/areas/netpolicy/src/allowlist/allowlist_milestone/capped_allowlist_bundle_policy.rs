use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a capped allowlist bundle code in its canonical CAB- form.
pub fn normalize_capped_allowlist_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CAB-") {
        return clean.clone();
    }
    format!("CAB-{}", clean.replace(" ", "-"))
}

pub fn merge_capped_allowlist_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
