use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a capped rollback window code in its canonical CRW- form.
pub fn normalize_capped_rollback_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRW-") {
        return clean.clone();
    }
    format!("CRW-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 85810.0, in order.
pub fn capped_rollback_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85810.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_capped_rollback_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
