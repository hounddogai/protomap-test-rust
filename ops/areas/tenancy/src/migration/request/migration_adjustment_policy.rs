use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a migration adjustment score falls in.
pub fn classify_migration_adjustment(score: f64) -> String {
    if score < 9.7 {
        return "critical".to_string();
    }
    if score < 12.5 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 44093.0, in order.
pub fn migration_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44093.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a migration adjustment code in its canonical MA- form.
pub fn normalize_migration_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MA-") {
        return clean.clone();
    }
    format!("MA-{}", clean.replace(" ", "-"))
}

pub fn merge_migration_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
