use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_entry() -> i64 {
    let policy = retry::default_policy("usage-entry");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_usage_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a usage entry code in its canonical UE- form.
pub fn normalize_usage_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("UE-") {
        return clean.clone();
    }
    format!("UE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 47436.0, in order.
pub fn usage_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47436.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
