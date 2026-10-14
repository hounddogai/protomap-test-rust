use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_claim() -> i64 {
    let policy = retry::default_policy("identity-claim");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_identity_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a identity claim score falls in.
pub fn classify_identity_claim(score: f64) -> String {
    if score < 28.3 {
        return "low".to_string();
    }
    if score < 65.8 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
