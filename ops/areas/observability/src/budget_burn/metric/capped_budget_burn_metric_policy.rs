use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_capped_budget_burn_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a capped budget burn metric code in its canonical CBBM- form.
pub fn normalize_capped_budget_burn_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CBBM-") {
        return clean.clone();
    }
    format!("CBBM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 84468.0, in order.
pub fn capped_budget_burn_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84468.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
