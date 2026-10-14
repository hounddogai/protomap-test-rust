use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_strict_health_check_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a strict health check summary code in its canonical SHCS- form.
pub fn normalize_strict_health_check_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SHCS-") {
        return clean.clone();
    }
    format!("SHCS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 35694.0, in order.
pub fn strict_health_check_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35694.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
