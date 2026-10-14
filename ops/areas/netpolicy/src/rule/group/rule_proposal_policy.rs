use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_proposal() -> i64 {
    let policy = retry::default_policy("rule-proposal");
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

pub fn merge_rule_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rule proposal score falls in.
pub fn classify_rule_proposal(score: f64) -> String {
    if score < 15.5 {
        return "elevated".to_string();
    }
    if score < 92.7 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
