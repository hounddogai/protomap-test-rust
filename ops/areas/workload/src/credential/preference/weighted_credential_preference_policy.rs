use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a weighted credential preference score falls in.
pub fn classify_weighted_credential_preference(score: f64) -> String {
    if score < 23.5 {
        return "ok".to_string();
    }
    if score < 49.0 {
        return "watch".to_string();
    }
    if score < 55.9 {
        return "medium".to_string();
    }
    if score < 89.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_credential_preference() -> i64 {
    let policy = retry::default_policy("weighted-credential-preference");
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

pub fn merge_weighted_credential_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
