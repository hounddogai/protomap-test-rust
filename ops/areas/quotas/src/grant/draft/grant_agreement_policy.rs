use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a grant agreement score falls in.
pub fn classify_grant_agreement(score: f64) -> String {
    if score < 4.4 {
        return "critical".to_string();
    }
    if score < 37.6 {
        return "high".to_string();
    }
    if score < 38.5 {
        return "watch".to_string();
    }
    if score < 66.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn merge_grant_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_grant_agreement() -> i64 {
    let policy = retry::default_policy("grant-agreement");
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
