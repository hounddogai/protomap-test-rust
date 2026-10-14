use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_strict_budget_burn_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a strict budget burn estimate score falls in.
pub fn classify_strict_budget_burn_estimate(score: f64) -> String {
    if score < 1.5 {
        return "minimal".to_string();
    }
    if score < 18.4 {
        return "medium".to_string();
    }
    if score < 23.1 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a strict budget burn estimate code in its canonical SBBE- form.
pub fn normalize_strict_budget_burn_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBBE-") {
        return clean.clone();
    }
    format!("SBBE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 81754.0, in order.
pub fn strict_budget_burn_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81754.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
