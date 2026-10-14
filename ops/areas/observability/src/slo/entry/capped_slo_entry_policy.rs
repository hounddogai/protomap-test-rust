use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 67417.0, in order.
pub fn capped_slo_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67417.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_slo_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a capped slo entry code in its canonical CSE- form.
pub fn normalize_capped_slo_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CSE-") {
        return clean.clone();
    }
    format!("CSE-{}", clean.replace(" ", "-"))
}

pub fn merge_capped_slo_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
