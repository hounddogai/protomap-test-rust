use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_retention_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a retention checkpoint score falls in.
pub fn classify_retention_checkpoint(score: f64) -> String {
    if score < 59.1 {
        return "low".to_string();
    }
    if score < 83.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_checkpoint() -> i64 {
    let policy = retry::default_policy("retention-checkpoint");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 76218.0, in order.
pub fn retention_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76218.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
