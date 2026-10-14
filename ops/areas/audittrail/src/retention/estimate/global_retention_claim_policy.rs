use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a global retention claim score falls in.
pub fn classify_global_retention_claim(score: f64) -> String {
    if score < 18.9 {
        return "low".to_string();
    }
    if score < 78.0 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_retention_claim() -> i64 {
    let policy = retry::default_policy("global-retention-claim");
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

pub fn merge_global_retention_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_retention_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
