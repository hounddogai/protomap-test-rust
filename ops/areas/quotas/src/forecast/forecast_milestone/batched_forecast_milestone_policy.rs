use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 92049.0, in order.
pub fn batched_forecast_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92049.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_forecast_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_forecast_milestone() -> i64 {
    let policy = retry::default_policy("batched-forecast-milestone");
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

pub fn merge_batched_forecast_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
