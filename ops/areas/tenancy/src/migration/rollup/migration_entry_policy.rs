use super::*;

use acme_kit::numbers;

/// Puts a migration entry code in its canonical ME- form.
pub fn normalize_migration_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ME-") {
        return clean.clone();
    }
    format!("ME-{}", clean.replace(" ", "-"))
}

pub fn migration_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn migration_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 14045.0, in order.
pub fn migration_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14045.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
