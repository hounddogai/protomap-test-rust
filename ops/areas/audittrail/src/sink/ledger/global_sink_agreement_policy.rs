use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_global_sink_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a global sink agreement code in its canonical GSA- form.
pub fn normalize_global_sink_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GSA-") {
        return clean.clone();
    }
    format!("GSA-{}", clean.replace(" ", "-"))
}
