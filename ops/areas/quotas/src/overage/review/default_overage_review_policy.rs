use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_default_overage_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default overage review code in its canonical DOR- form.
pub fn normalize_default_overage_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DOR-") {
        return clean.clone();
    }
    format!("DOR-{}", clean.replace(" ", "-"))
}
