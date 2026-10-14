use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn global_release_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn merge_global_release_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a global release entry code in its canonical GRE- form.
pub fn normalize_global_release_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRE-") {
        return clean.clone();
    }
    format!("GRE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 60611.0, in order.
pub fn global_release_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60611.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
