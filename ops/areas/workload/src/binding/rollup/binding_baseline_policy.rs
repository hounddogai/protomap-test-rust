use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 8699.0, in order.
pub fn binding_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8699.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a binding baseline score falls in.
pub fn classify_binding_baseline(score: f64) -> String {
    if score < 56.9 {
        return "minimal".to_string();
    }
    if score < 78.6 {
        return "watch".to_string();
    }
    if score < 79.0 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_binding_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_binding_baseline() -> i64 {
    let policy = retry::default_policy("binding-baseline");
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
