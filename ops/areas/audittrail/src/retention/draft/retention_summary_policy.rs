use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_retention_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a retention summary score falls in.
pub fn classify_retention_summary(score: f64) -> String {
    if score < 7.2 {
        return "minimal".to_string();
    }
    if score < 25.6 {
        return "watch".to_string();
    }
    if score < 92.4 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_summary() -> i64 {
    let policy = retry::default_policy("retention-summary");
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
