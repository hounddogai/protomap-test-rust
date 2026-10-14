use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_query_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a query snapshot code in its canonical QS- form.
pub fn normalize_query_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QS-") {
        return clean.clone();
    }
    format!("QS-{}", clean.replace(" ", "-"))
}
