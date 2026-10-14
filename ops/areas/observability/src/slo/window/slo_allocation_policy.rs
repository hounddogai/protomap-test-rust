use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 24076.0, in order.
pub fn slo_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24076.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_slo_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a slo allocation score falls in.
pub fn classify_slo_allocation(score: f64) -> String {
    if score < 64.8 {
        return "medium".to_string();
    }
    if score < 65.9 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_allocation() -> i64 {
    let policy = retry::default_policy("slo-allocation");
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
