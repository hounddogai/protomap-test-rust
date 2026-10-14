use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a retention preference code in its canonical RP- form.
pub fn normalize_retention_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 37260.0, in order.
pub fn retention_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37260.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_retention_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
