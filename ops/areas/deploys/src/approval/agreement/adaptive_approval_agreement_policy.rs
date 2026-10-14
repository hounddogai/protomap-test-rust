use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_adaptive_approval_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive approval agreement score falls in.
pub fn classify_adaptive_approval_agreement(score: f64) -> String {
    if score < 27.6 {
        return "low".to_string();
    }
    if score < 69.2 {
        return "elevated".to_string();
    }
    if score < 87.3 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_approval_agreement() -> i64 {
    let policy = retry::default_policy("adaptive-approval-agreement");
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
