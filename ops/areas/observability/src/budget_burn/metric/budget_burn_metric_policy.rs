use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_metric() -> i64 {
    let policy = retry::default_policy("budget-burn-metric");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_budget_burn_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a budget burn metric score falls in.
pub fn classify_budget_burn_metric(score: f64) -> String {
    if score < 11.4 {
        return "high".to_string();
    }
    if score < 69.8 {
        return "critical".to_string();
    }
    "ok".to_string()
}
