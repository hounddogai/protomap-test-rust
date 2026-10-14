use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a query claim score falls in.
pub fn classify_query_claim(score: f64) -> String {
    if score < 16.1 {
        return "critical".to_string();
    }
    if score < 59.9 {
        return "minimal".to_string();
    }
    if score < 63.3 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_query_claim() -> i64 {
    let policy = retry::default_policy("query-claim");
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

pub fn merge_query_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
