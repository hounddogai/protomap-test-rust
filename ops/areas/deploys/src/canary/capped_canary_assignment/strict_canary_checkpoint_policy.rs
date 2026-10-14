use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 91525.0, in order.
pub fn strict_canary_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91525.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn strict_canary_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a strict canary checkpoint code in its canonical SCC- form.
pub fn normalize_strict_canary_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SCC-") {
        return clean.clone();
    }
    format!("SCC-{}", clean.replace(" ", "-"))
}

pub fn merge_strict_canary_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
