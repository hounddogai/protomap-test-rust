use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn weighted_trust_domain_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_weighted_trust_domain_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a weighted trust domain baseline score falls in.
pub fn classify_weighted_trust_domain_baseline(score: f64) -> String {
    if score < 72.3 {
        return "watch".to_string();
    }
    if score < 73.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_trust_domain_baseline() -> i64 {
    let policy = retry::default_policy("weighted-trust-domain-baseline");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
