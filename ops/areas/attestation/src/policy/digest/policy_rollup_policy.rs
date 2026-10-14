use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a policy rollup score falls in.
pub fn classify_policy_rollup(score: f64) -> String {
    if score < 19.6 {
        return "critical".to_string();
    }
    if score < 38.6 {
        return "ok".to_string();
    }
    if score < 83.7 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 91499.0, in order.
pub fn policy_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91499.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn policy_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_policy_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
