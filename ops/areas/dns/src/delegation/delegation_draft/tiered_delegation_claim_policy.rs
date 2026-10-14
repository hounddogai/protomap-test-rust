use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_tiered_delegation_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered delegation claim code in its canonical TDC- form.
pub fn normalize_tiered_delegation_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDC-") {
        return clean.clone();
    }
    format!("TDC-{}", clean.replace(" ", "-"))
}

pub fn tiered_delegation_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn tiered_delegation_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
