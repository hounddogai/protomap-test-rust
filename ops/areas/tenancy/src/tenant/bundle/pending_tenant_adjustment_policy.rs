use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a pending tenant adjustment code in its canonical PTA- form.
pub fn normalize_pending_tenant_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTA-") {
        return clean.clone();
    }
    format!("PTA-{}", clean.replace(" ", "-"))
}

pub fn merge_pending_tenant_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
