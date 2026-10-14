use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 17667.0, in order.
pub fn probe_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17667.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_probe_template() -> i64 {
    let policy = retry::default_policy("probe-template");
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

/// Names the band a probe template score falls in.
pub fn classify_probe_template(score: f64) -> String {
    if score < 20.2 {
        return "watch".to_string();
    }
    if score < 64.3 {
        return "low".to_string();
    }
    if score < 73.9 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn merge_probe_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
