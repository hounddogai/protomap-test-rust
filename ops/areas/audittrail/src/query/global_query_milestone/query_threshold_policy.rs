use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a query threshold score falls in.
pub fn classify_query_threshold(score: f64) -> String {
    if score < 4.4 {
        return "medium".to_string();
    }
    if score < 8.9 {
        return "ok".to_string();
    }
    if score < 24.2 {
        return "low".to_string();
    }
    if score < 58.7 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn merge_query_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_query_threshold() -> i64 {
    let policy = retry::default_policy("query-threshold");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn query_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.73;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
