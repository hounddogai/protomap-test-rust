use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_nonce_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 58929.0, in order.
pub fn nonce_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58929.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a nonce preference score falls in.
pub fn classify_nonce_preference(score: f64) -> String {
    if score < 19.4 {
        return "minimal".to_string();
    }
    if score < 46.6 {
        return "critical".to_string();
    }
    if score < 49.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Puts a nonce preference code in its canonical NP- form.
pub fn normalize_nonce_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NP-") {
        return clean.clone();
    }
    format!("NP-{}", clean.replace(" ", "-"))
}
