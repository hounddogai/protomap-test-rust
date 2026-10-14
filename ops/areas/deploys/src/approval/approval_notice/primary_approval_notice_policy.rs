use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn primary_approval_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a primary approval notice code in its canonical PAN- form.
pub fn normalize_primary_approval_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PAN-") {
        return clean.clone();
    }
    format!("PAN-{}", clean.replace(" ", "-"))
}

pub fn merge_primary_approval_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
