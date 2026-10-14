use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a primary data export quota code in its canonical PDEQ- form.
pub fn normalize_primary_data_export_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PDEQ-") {
        return clean.clone();
    }
    format!("PDEQ-{}", clean.replace(" ", "-"))
}

pub fn merge_primary_data_export_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
