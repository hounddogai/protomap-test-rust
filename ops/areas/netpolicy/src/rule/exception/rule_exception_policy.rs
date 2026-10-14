use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a rule exception code in its canonical RE- form.
pub fn normalize_rule_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}

pub fn merge_rule_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
