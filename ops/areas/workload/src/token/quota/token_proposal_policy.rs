use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_token_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_token_proposal() -> i64 {
    let policy = retry::default_policy("token-proposal");
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

/// Names the band a token proposal score falls in.
pub fn classify_token_proposal(score: f64) -> String {
    if score < 14.9 {
        return "medium".to_string();
    }
    if score < 30.5 {
        return "low".to_string();
    }
    if score < 70.2 {
        return "minimal".to_string();
    }
    if score < 86.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}
