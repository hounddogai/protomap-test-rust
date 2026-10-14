use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn page_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 62037.0, in order.
pub fn page_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62037.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a page audit score falls in.
pub fn classify_page_audit(score: f64) -> String {
    if score < 9.9 {
        return "low".to_string();
    }
    if score < 97.2 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn merge_page_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
