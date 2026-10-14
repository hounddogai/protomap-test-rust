use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_rolling_nonce_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 93739.0, in order.
pub fn rolling_nonce_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93739.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling nonce budget code in its canonical RNB- form.
pub fn normalize_rolling_nonce_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RNB-") {
        return clean.clone();
    }
    format!("RNB-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling nonce budget score falls in.
pub fn classify_rolling_nonce_budget(score: f64) -> String {
    if score < 49.3 {
        return "high".to_string();
    }
    if score < 69.8 {
        return "minimal".to_string();
    }
    if score < 86.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
