use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a tiered delegation request code in its canonical TDR- form.
pub fn normalize_tiered_delegation_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDR-") {
        return clean.clone();
    }
    format!("TDR-{}", clean.replace(" ", "-"))
}

pub fn merge_tiered_delegation_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
