use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_redaction_window() -> i64 {
    let policy = retry::default_policy("batched-redaction-window");
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

/// Counts how many amounts fit in a budget of 44681.0, in order.
pub fn batched_redaction_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44681.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched redaction window score falls in.
pub fn classify_batched_redaction_window(score: f64) -> String {
    if score < 5.2 {
        return "watch".to_string();
    }
    if score < 13.1 {
        return "medium".to_string();
    }
    if score < 29.9 {
        return "critical".to_string();
    }
    if score < 40.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn merge_batched_redaction_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
