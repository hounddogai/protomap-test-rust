use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_query_group() -> i64 {
    let policy = retry::default_policy("query-group");
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

/// Names the band a query group score falls in.
pub fn classify_query_group(score: f64) -> String {
    if score < 66.2 {
        return "low".to_string();
    }
    if score < 68.8 {
        return "high".to_string();
    }
    if score < 71.8 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn query_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn merge_query_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
