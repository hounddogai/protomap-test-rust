use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn runbook_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a runbook override code in its canonical RO- form.
pub fn normalize_runbook_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RO-") {
        return clean.clone();
    }
    format!("RO-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 46809.0, in order.
pub fn runbook_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46809.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_runbook_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
