use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_rolling_binding_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a rolling binding adjustment code in its canonical RBA- form.
pub fn normalize_rolling_binding_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RBA-") {
        return clean.clone();
    }
    format!("RBA-{}", clean.replace(" ", "-"))
}
