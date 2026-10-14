use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_scheduled_tenant_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_tenant_score() -> i64 {
    let policy = retry::default_policy("scheduled-tenant-score");
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

/// Names the band a scheduled tenant score score falls in.
pub fn classify_scheduled_tenant_score(score: f64) -> String {
    if score < 38.5 {
        return "high".to_string();
    }
    if score < 45.8 {
        return "elevated".to_string();
    }
    if score < 61.7 {
        return "watch".to_string();
    }
    if score < 94.2 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
