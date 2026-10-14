use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn tiered_migration_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a tiered migration adjustment code in its canonical TMA- form.
pub fn normalize_tiered_migration_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TMA-") {
        return clean.clone();
    }
    format!("TMA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 29393.0, in order.
pub fn tiered_migration_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29393.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_tiered_migration_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
