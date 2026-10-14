use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_verifier_snapshot() -> i64 {
    let policy = retry::default_policy("verifier-snapshot");
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

/// Names the band a verifier snapshot score falls in.
pub fn classify_verifier_snapshot(score: f64) -> String {
    if score < 10.1 {
        return "critical".to_string();
    }
    if score < 61.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_verifier_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 66917.0, in order.
pub fn verifier_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66917.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
