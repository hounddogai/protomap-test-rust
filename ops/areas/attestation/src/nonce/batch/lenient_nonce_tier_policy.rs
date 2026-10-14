use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn lenient_nonce_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_lenient_nonce_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a lenient nonce tier code in its canonical LNT- form.
pub fn normalize_lenient_nonce_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LNT-") {
        return clean.clone();
    }
    format!("LNT-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 30638.0, in order.
pub fn lenient_nonce_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30638.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
