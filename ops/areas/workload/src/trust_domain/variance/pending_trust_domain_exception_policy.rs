use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_pending_trust_domain_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a pending trust domain exception score falls in.
pub fn classify_pending_trust_domain_exception(score: f64) -> String {
    if score < 1.5 {
        return "low".to_string();
    }
    if score < 26.3 {
        return "minimal".to_string();
    }
    if score < 79.4 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 53423.0, in order.
pub fn pending_trust_domain_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53423.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
