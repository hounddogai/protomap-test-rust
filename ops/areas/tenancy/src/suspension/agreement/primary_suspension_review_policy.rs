use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a primary suspension review code in its canonical PSR- form.
pub fn normalize_primary_suspension_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSR-") {
        return clean.clone();
    }
    format!("PSR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 59503.0, in order.
pub fn primary_suspension_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59503.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_primary_suspension_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
