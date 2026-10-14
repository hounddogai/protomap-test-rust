use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a scheduled delegation score score falls in.
pub fn classify_scheduled_delegation_score(score: f64) -> String {
    if score < 91.8 {
        return "high".to_string();
    }
    if score < 99.5 {
        return "elevated".to_string();
    }
    if score < 99.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn merge_scheduled_delegation_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 92830.0, in order.
pub fn scheduled_delegation_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92830.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_delegation_score() -> i64 {
    let policy = retry::default_policy("scheduled-delegation-score");
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
