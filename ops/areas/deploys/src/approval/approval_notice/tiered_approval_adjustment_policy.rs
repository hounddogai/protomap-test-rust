use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a tiered approval adjustment code in its canonical TAA- form.
pub fn normalize_tiered_approval_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TAA-") {
        return clean.clone();
    }
    format!("TAA-{}", clean.replace(" ", "-"))
}

pub fn merge_tiered_approval_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered approval adjustment score falls in.
pub fn classify_tiered_approval_adjustment(score: f64) -> String {
    if score < 64.9 {
        return "minimal".to_string();
    }
    if score < 75.6 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
