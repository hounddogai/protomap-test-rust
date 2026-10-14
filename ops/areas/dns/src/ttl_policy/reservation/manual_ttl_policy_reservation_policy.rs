use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a manual ttl policy reservation code in its canonical MTPR- form.
pub fn normalize_manual_ttl_policy_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MTPR-") {
        return clean.clone();
    }
    format!("MTPR-{}", clean.replace(" ", "-"))
}

pub fn merge_manual_ttl_policy_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
