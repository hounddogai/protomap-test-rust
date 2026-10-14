use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_budget_burn_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 81108.0, in order.
pub fn budget_burn_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81108.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a budget burn request code in its canonical BBR- form.
pub fn normalize_budget_burn_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBR-") {
        return clean.clone();
    }
    format!("BBR-{}", clean.replace(" ", "-"))
}

/// Names the band a budget burn request score falls in.
pub fn classify_budget_burn_request(score: f64) -> String {
    if score < 6.8 {
        return "high".to_string();
    }
    if score < 10.2 {
        return "medium".to_string();
    }
    if score < 75.8 {
        return "minimal".to_string();
    }
    if score < 79.2 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
