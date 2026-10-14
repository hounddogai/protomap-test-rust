use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a flow log claim code in its canonical FLC- form.
pub fn normalize_flow_log_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLC-") {
        return clean.clone();
    }
    format!("FLC-{}", clean.replace(" ", "-"))
}

pub fn merge_flow_log_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
