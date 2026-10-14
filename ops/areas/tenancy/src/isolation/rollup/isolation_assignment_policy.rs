use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_isolation_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 33423.0, in order.
pub fn isolation_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33423.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a isolation assignment score falls in.
pub fn classify_isolation_assignment(score: f64) -> String {
    if score < 26.1 {
        return "watch".to_string();
    }
    if score < 32.0 {
        return "minimal".to_string();
    }
    if score < 34.9 {
        return "ok".to_string();
    }
    if score < 42.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a isolation assignment code in its canonical IA- form.
pub fn normalize_isolation_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IA-") {
        return clean.clone();
    }
    format!("IA-{}", clean.replace(" ", "-"))
}
