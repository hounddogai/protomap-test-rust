use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a isolation review code in its canonical IR- form.
pub fn normalize_isolation_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IR-") {
        return clean.clone();
    }
    format!("IR-{}", clean.replace(" ", "-"))
}

pub fn merge_isolation_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 20040.0, in order.
pub fn isolation_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20040.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
