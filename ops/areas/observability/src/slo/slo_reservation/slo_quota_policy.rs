use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a slo quota score falls in.
pub fn classify_slo_quota(score: f64) -> String {
    if score < 26.3 {
        return "ok".to_string();
    }
    if score < 57.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 47949.0, in order.
pub fn slo_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47949.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_quota() -> i64 {
    let policy = retry::default_policy("slo-quota");
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

pub fn merge_slo_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
