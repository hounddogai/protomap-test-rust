use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 34698.0, in order.
pub fn default_identity_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 34698.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_default_identity_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default identity score code in its canonical DIS- form.
pub fn normalize_default_identity_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DIS-") {
        return clean.clone();
    }
    format!("DIS-{}", clean.replace(" ", "-"))
}
