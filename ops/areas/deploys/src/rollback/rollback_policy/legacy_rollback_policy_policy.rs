use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_legacy_rollback_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn legacy_rollback_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a legacy rollback policy code in its canonical LRP- form.
pub fn normalize_legacy_rollback_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRP-") {
        return clean.clone();
    }
    format!("LRP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 85907.0, in order.
pub fn legacy_rollback_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85907.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
