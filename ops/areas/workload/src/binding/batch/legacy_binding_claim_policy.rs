use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_binding_claim() -> i64 {
    let policy = retry::default_policy("legacy-binding-claim");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn legacy_binding_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a legacy binding claim score falls in.
pub fn classify_legacy_binding_claim(score: f64) -> String {
    if score < 36.7 {
        return "elevated".to_string();
    }
    if score < 38.8 {
        return "watch".to_string();
    }
    if score < 51.3 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn merge_legacy_binding_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
