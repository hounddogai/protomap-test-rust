use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_regional_query_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_query_rollup() -> i64 {
    let policy = retry::default_policy("regional-query-rollup");
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

/// Counts how many amounts fit in a budget of 79896.0, in order.
pub fn regional_query_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79896.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional query rollup score falls in.
pub fn classify_regional_query_rollup(score: f64) -> String {
    if score < 76.6 {
        return "watch".to_string();
    }
    if score < 82.6 {
        return "low".to_string();
    }
    if score < 95.6 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
