use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a rolling page entry code in its canonical RPE- form.
pub fn normalize_rolling_page_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPE-") {
        return clean.clone();
    }
    format!("RPE-{}", clean.replace(" ", "-"))
}

pub fn merge_rolling_page_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
