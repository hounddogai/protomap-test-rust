use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_page_plan() -> i64 {
    let policy = retry::default_policy("legacy-page-plan");
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

pub fn legacy_page_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_legacy_page_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy page plan score falls in.
pub fn classify_legacy_page_plan(score: f64) -> String {
    if score < 96.0 {
        return "critical".to_string();
    }
    if score < 96.2 {
        return "watch".to_string();
    }
    if score < 99.5 {
        return "high".to_string();
    }
    "elevated".to_string()
}
