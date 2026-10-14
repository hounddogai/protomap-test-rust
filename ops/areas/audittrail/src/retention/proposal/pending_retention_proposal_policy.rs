use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_retention_proposal() -> i64 {
    let policy = retry::default_policy("pending-retention-proposal");
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

pub fn merge_pending_retention_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a pending retention proposal score falls in.
pub fn classify_pending_retention_proposal(score: f64) -> String {
    if score < 8.5 {
        return "low".to_string();
    }
    if score < 42.6 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
