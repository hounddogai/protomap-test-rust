use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_template() -> i64 {
    let policy = retry::default_policy("isolation-template");
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

pub fn merge_isolation_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn isolation_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a isolation template score falls in.
pub fn classify_isolation_template(score: f64) -> String {
    if score < 47.6 {
        return "elevated".to_string();
    }
    if score < 54.3 {
        return "medium".to_string();
    }
    if score < 77.5 {
        return "minimal".to_string();
    }
    if score < 78.0 {
        return "watch".to_string();
    }
    "high".to_string()
}
