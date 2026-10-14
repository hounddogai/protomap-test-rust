use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a tiered slo manifest code in its canonical TSM- form.
pub fn normalize_tiered_slo_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TSM-") {
        return clean.clone();
    }
    format!("TSM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 89437.0, in order.
pub fn tiered_slo_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89437.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_tiered_slo_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
