use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_trust_domain_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a trust domain manifest code in its canonical TDM- form.
pub fn normalize_trust_domain_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDM-") {
        return clean.clone();
    }
    format!("TDM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 63544.0, in order.
pub fn trust_domain_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63544.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn trust_domain_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
