use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a batched overage rule score falls in.
pub fn classify_batched_overage_rule(score: f64) -> String {
    if score < 22.7 {
        return "critical".to_string();
    }
    if score < 24.6 {
        return "ok".to_string();
    }
    if score < 47.8 {
        return "watch".to_string();
    }
    if score < 61.9 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn merge_batched_overage_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_overage_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 99606.0, in order.
pub fn batched_overage_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99606.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
