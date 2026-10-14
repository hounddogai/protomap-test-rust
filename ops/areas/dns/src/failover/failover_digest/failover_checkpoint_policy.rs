use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_failover_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a failover checkpoint code in its canonical FC- form.
pub fn normalize_failover_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FC-") {
        return clean.clone();
    }
    format!("FC-{}", clean.replace(" ", "-"))
}
