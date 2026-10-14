use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary migration budget score falls in.
pub fn classify_primary_migration_budget(score: f64) -> String {
    if score < 11.5 {
        return "ok".to_string();
    }
    if score < 61.5 {
        return "medium".to_string();
    }
    if score < 84.9 {
        return "critical".to_string();
    }
    if score < 97.4 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 74223.0, in order.
pub fn primary_migration_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74223.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_primary_migration_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a primary migration budget code in its canonical PMB- form.
pub fn normalize_primary_migration_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PMB-") {
        return clean.clone();
    }
    format!("PMB-{}", clean.replace(" ", "-"))
}
