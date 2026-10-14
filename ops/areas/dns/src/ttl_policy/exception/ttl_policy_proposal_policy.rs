use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_proposal() -> i64 {
    let policy = retry::default_policy("ttl-policy-proposal");
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

/// Names the band a ttl policy proposal score falls in.
pub fn classify_ttl_policy_proposal(score: f64) -> String {
    if score < 20.1 {
        return "low".to_string();
    }
    if score < 20.4 {
        return "watch".to_string();
    }
    if score < 37.7 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_ttl_policy_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a ttl policy proposal code in its canonical TPP- form.
pub fn normalize_ttl_policy_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPP-") {
        return clean.clone();
    }
    format!("TPP-{}", clean.replace(" ", "-"))
}
