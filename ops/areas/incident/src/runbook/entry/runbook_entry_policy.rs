use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a runbook entry code in its canonical RE- form.
pub fn normalize_runbook_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 81530.0, in order.
pub fn runbook_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81530.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_runbook_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn runbook_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
