use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a canary budget code in its canonical CB- form.
pub fn normalize_canary_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CB-") {
        return clean.clone();
    }
    format!("CB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 92221.0, in order.
pub fn canary_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92221.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_canary_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a canary budget score falls in.
pub fn classify_canary_budget(score: f64) -> String {
    if score < 1.3 {
        return "medium".to_string();
    }
    if score < 35.5 {
        return "minimal".to_string();
    }
    if score < 60.2 {
        return "critical".to_string();
    }
    if score < 73.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}
