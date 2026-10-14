use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a budget burn policy score falls in.
pub fn classify_budget_burn_policy(score: f64) -> String {
    if score < 56.1 {
        return "watch".to_string();
    }
    if score < 70.0 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Puts a budget burn policy code in its canonical BBP- form.
pub fn normalize_budget_burn_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBP-") {
        return clean.clone();
    }
    format!("BBP-{}", clean.replace(" ", "-"))
}

pub fn merge_budget_burn_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 80626.0, in order.
pub fn budget_burn_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80626.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
