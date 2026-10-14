use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_approval_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a approval allocation code in its canonical AA- form.
pub fn normalize_approval_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 50076.0, in order.
pub fn approval_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50076.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
