use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 3984.0, in order.
pub fn freeze_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3984.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_freeze_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a freeze milestone code in its canonical FM- form.
pub fn normalize_freeze_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FM-") {
        return clean.clone();
    }
    format!("FM-{}", clean.replace(" ", "-"))
}
