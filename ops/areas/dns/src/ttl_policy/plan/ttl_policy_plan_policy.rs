use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a ttl policy plan code in its canonical TPP- form.
pub fn normalize_ttl_policy_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPP-") {
        return clean.clone();
    }
    format!("TPP-{}", clean.replace(" ", "-"))
}

pub fn merge_ttl_policy_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 58847.0, in order.
pub fn ttl_policy_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58847.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn ttl_policy_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
