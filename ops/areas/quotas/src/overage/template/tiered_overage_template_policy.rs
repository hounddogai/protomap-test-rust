use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_overage_template() -> i64 {
    let policy = retry::default_policy("tiered-overage-template");
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

pub fn merge_tiered_overage_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn tiered_overage_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a tiered overage template score falls in.
pub fn classify_tiered_overage_template(score: f64) -> String {
    if score < 44.9 {
        return "medium".to_string();
    }
    if score < 89.3 {
        return "critical".to_string();
    }
    "ok".to_string()
}
