use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a pending probe override code in its canonical PPO- form.
pub fn normalize_pending_probe_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPO-") {
        return clean.clone();
    }
    format!("PPO-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 87365.0, in order.
pub fn pending_probe_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87365.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_probe_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn merge_pending_probe_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
