use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a regional runbook entry code in its canonical RRE- form.
pub fn normalize_regional_runbook_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRE-") {
        return clean.clone();
    }
    format!("RRE-{}", clean.replace(" ", "-"))
}

pub fn merge_regional_runbook_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
