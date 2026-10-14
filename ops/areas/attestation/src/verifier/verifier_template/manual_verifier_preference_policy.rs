use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn manual_verifier_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn merge_manual_verifier_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 22434.0, in order.
pub fn manual_verifier_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 22434.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a manual verifier preference code in its canonical MVP- form.
pub fn normalize_manual_verifier_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MVP-") {
        return clean.clone();
    }
    format!("MVP-{}", clean.replace(" ", "-"))
}
