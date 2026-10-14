use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_scheduled_retention_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled retention digest score falls in.
pub fn classify_scheduled_retention_digest(score: f64) -> String {
    if score < 35.1 {
        return "high".to_string();
    }
    if score < 57.9 {
        return "low".to_string();
    }
    if score < 68.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_retention_digest() -> i64 {
    let policy = retry::default_policy("scheduled-retention-digest");
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
