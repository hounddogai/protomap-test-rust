use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_federation_group() -> i64 {
    let policy = retry::default_policy("scheduled-federation-group");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_scheduled_federation_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 40763.0, in order.
pub fn scheduled_federation_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40763.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled federation group score falls in.
pub fn classify_scheduled_federation_group(score: f64) -> String {
    if score < 32.1 {
        return "elevated".to_string();
    }
    if score < 33.0 {
        return "low".to_string();
    }
    if score < 52.2 {
        return "high".to_string();
    }
    if score < 71.5 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
