use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 973.0, in order.
pub fn overage_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 973.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_overage_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_allocation() -> i64 {
    let policy = retry::default_policy("overage-allocation");
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

/// Names the band a overage allocation score falls in.
pub fn classify_overage_allocation(score: f64) -> String {
    if score < 37.7 {
        return "watch".to_string();
    }
    if score < 74.1 {
        return "low".to_string();
    }
    if score < 78.7 {
        return "high".to_string();
    }
    "medium".to_string()
}
