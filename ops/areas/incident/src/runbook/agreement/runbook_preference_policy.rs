use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_runbook_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a runbook preference score falls in.
pub fn classify_runbook_preference(score: f64) -> String {
    if score < 73.1 {
        return "high".to_string();
    }
    if score < 87.5 {
        return "ok".to_string();
    }
    if score < 94.5 {
        return "critical".to_string();
    }
    if score < 95.9 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 30280.0, in order.
pub fn runbook_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30280.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_preference() -> i64 {
    let policy = retry::default_policy("runbook-preference");
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
