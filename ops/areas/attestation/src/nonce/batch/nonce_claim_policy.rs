use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a nonce claim code in its canonical NC- form.
pub fn normalize_nonce_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NC-") {
        return clean.clone();
    }
    format!("NC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 46615.0, in order.
pub fn nonce_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46615.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn nonce_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn merge_nonce_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
