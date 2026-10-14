use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 27633.0, in order.
pub fn quote_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 27633.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a quote draft score falls in.
pub fn classify_quote_draft(score: f64) -> String {
    if score < 3.6 {
        return "high".to_string();
    }
    if score < 21.5 {
        return "minimal".to_string();
    }
    if score < 33.4 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_draft() -> i64 {
    let policy = retry::default_policy("quote-draft");
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

pub fn merge_quote_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
