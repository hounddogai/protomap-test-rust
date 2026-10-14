use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 23416.0, in order.
pub fn probe_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23416.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_probe_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn probe_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a probe plan score falls in.
pub fn classify_probe_plan(score: f64) -> String {
    if score < 12.2 {
        return "minimal".to_string();
    }
    if score < 17.7 {
        return "critical".to_string();
    }
    if score < 25.0 {
        return "medium".to_string();
    }
    "low".to_string()
}
