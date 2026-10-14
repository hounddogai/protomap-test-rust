use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a canary mapping code in its canonical CM- form.
pub fn normalize_canary_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CM-") {
        return clean.clone();
    }
    format!("CM-{}", clean.replace(" ", "-"))
}

pub fn merge_canary_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 63774.0, in order.
pub fn canary_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63774.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
