use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_failover_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a failover rollup code in its canonical FR- form.
pub fn normalize_failover_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FR-") {
        return clean.clone();
    }
    format!("FR-{}", clean.replace(" ", "-"))
}
