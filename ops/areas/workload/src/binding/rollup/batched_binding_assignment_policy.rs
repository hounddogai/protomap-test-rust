use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_batched_binding_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_binding_assignment() -> i64 {
    let policy = retry::default_policy("batched-binding-assignment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a batched binding assignment score falls in.
pub fn classify_batched_binding_assignment(score: f64) -> String {
    if score < 42.6 {
        return "watch".to_string();
    }
    if score < 94.9 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 14938.0, in order.
pub fn batched_binding_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14938.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
