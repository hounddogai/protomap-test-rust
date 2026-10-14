use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a tiered token claim code in its canonical TTC- form.
pub fn normalize_tiered_token_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTC-") {
        return clean.clone();
    }
    format!("TTC-{}", clean.replace(" ", "-"))
}

pub fn merge_tiered_token_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
