use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_strict_flow_log_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a strict flow log checkpoint code in its canonical SFLC- form.
pub fn normalize_strict_flow_log_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFLC-") {
        return clean.clone();
    }
    format!("SFLC-{}", clean.replace(" ", "-"))
}
