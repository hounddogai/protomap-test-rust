use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_summary() -> i64 {
    let policy = retry::default_policy("budget-burn-summary");
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

pub fn merge_budget_burn_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a budget burn summary score falls in.
pub fn classify_budget_burn_summary(score: f64) -> String {
    if score < 29.0 {
        return "critical".to_string();
    }
    if score < 55.7 {
        return "high".to_string();
    }
    if score < 79.6 {
        return "minimal".to_string();
    }
    if score < 80.4 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn budget_burn_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
