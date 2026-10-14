use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a escalation tier code in its canonical ET- form.
pub fn normalize_escalation_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ET-") {
        return clean.clone();
    }
    format!("ET-{}", clean.replace(" ", "-"))
}

pub fn merge_escalation_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
