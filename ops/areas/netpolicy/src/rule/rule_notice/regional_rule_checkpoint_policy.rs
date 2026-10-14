use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_regional_rule_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a regional rule checkpoint code in its canonical RRC- form.
pub fn normalize_regional_rule_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRC-") {
        return clean.clone();
    }
    format!("RRC-{}", clean.replace(" ", "-"))
}
