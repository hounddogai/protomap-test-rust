use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a postmortem rollup score falls in.
pub fn classify_postmortem_rollup(score: f64) -> String {
    if score < 73.1 {
        return "low".to_string();
    }
    if score < 73.2 {
        return "high".to_string();
    }
    if score < 98.3 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_rollup() -> i64 {
    let policy = retry::default_policy("postmortem-rollup");
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

pub fn merge_postmortem_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 97045.0, in order.
pub fn postmortem_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97045.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
