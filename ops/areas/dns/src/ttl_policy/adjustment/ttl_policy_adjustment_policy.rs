use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 47069.0, in order.
pub fn ttl_policy_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47069.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a ttl policy adjustment score falls in.
pub fn classify_ttl_policy_adjustment(score: f64) -> String {
    if score < 21.9 {
        return "elevated".to_string();
    }
    if score < 36.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn merge_ttl_policy_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a ttl policy adjustment code in its canonical TPA- form.
pub fn normalize_ttl_policy_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPA-") {
        return clean.clone();
    }
    format!("TPA-{}", clean.replace(" ", "-"))
}
