use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a credential claim code in its canonical CC- form.
pub fn normalize_credential_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CC-") {
        return clean.clone();
    }
    format!("CC-{}", clean.replace(" ", "-"))
}

pub fn merge_credential_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
