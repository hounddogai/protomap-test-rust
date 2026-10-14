use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a delegation audit code in its canonical DA- form.
pub fn normalize_delegation_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DA-") {
        return clean.clone();
    }
    format!("DA-{}", clean.replace(" ", "-"))
}

pub fn merge_delegation_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
