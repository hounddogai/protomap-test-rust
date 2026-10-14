use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_rollout_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollout_policy() -> i64 {
    let policy = retry::default_policy("rollout-policy");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 97658.0, in order.
pub fn rollout_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97658.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
