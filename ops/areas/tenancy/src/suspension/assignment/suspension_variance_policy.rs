use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn suspension_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a suspension variance code in its canonical SV- form.
pub fn normalize_suspension_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SV-") {
        return clean.clone();
    }
    format!("SV-{}", clean.replace(" ", "-"))
}

pub fn merge_suspension_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 86150.0, in order.
pub fn suspension_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86150.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
