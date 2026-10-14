use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 92628.0, in order.
pub fn freeze_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92628.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a freeze budget code in its canonical FB- form.
pub fn normalize_freeze_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FB-") {
        return clean.clone();
    }
    format!("FB-{}", clean.replace(" ", "-"))
}

pub fn merge_freeze_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
