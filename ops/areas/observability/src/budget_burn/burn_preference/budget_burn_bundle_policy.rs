use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn budget_burn_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 48314.0, in order.
pub fn budget_burn_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48314.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_budget_burn_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a budget burn bundle score falls in.
pub fn classify_budget_burn_bundle(score: f64) -> String {
    if score < 33.4 {
        return "elevated".to_string();
    }
    if score < 44.2 {
        return "watch".to_string();
    }
    "ok".to_string()
}
