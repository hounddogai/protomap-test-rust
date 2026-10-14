use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn audit_event_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn merge_audit_event_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a audit event segment code in its canonical AES- form.
pub fn normalize_audit_event_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AES-") {
        return clean.clone();
    }
    format!("AES-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 8098.0, in order.
pub fn audit_event_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8098.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
