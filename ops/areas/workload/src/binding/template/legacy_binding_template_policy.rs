use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_legacy_binding_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_binding_template() -> i64 {
    let policy = retry::default_policy("legacy-binding-template");
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

/// Names the band a legacy binding template score falls in.
pub fn classify_legacy_binding_template(score: f64) -> String {
    if score < 31.7 {
        return "watch".to_string();
    }
    if score < 46.1 {
        return "critical".to_string();
    }
    if score < 87.9 {
        return "minimal".to_string();
    }
    if score < 99.9 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 8098.0, in order.
pub fn legacy_binding_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8098.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
