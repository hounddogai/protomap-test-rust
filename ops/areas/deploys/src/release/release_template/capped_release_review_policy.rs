use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn capped_release_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a capped release review score falls in.
pub fn classify_capped_release_review(score: f64) -> String {
    if score < 72.3 {
        return "medium".to_string();
    }
    if score < 95.2 {
        return "critical".to_string();
    }
    if score < 98.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_capped_release_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_release_review() -> i64 {
    let policy = retry::default_policy("capped-release-review");
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
