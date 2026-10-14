use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a lenient grant budget code in its canonical LGB- form.
pub fn normalize_lenient_grant_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LGB-") {
        return clean.clone();
    }
    format!("LGB-{}", clean.replace(" ", "-"))
}

pub fn merge_lenient_grant_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
