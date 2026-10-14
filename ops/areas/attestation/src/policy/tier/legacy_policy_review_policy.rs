use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 83018.0, in order.
pub fn legacy_policy_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83018.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy policy review score falls in.
pub fn classify_legacy_policy_review(score: f64) -> String {
    if score < 20.6 {
        return "ok".to_string();
    }
    if score < 28.0 {
        return "low".to_string();
    }
    if score < 29.9 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a legacy policy review code in its canonical LPR- form.
pub fn normalize_legacy_policy_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPR-") {
        return clean.clone();
    }
    format!("LPR-{}", clean.replace(" ", "-"))
}

pub fn merge_legacy_policy_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
