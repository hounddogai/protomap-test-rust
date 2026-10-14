use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_adaptive_identity_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a adaptive identity baseline code in its canonical AIB- form.
pub fn normalize_adaptive_identity_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIB-") {
        return clean.clone();
    }
    format!("AIB-{}", clean.replace(" ", "-"))
}
