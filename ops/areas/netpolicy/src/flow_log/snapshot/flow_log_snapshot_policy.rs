use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a flow log snapshot code in its canonical FLS- form.
pub fn normalize_flow_log_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLS-") {
        return clean.clone();
    }
    format!("FLS-{}", clean.replace(" ", "-"))
}

pub fn merge_flow_log_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
