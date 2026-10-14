use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_lenient_plan_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_plan_score() -> i64 {
    let policy = retry::default_policy("lenient-plan-score");
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

/// Names the band a lenient plan score score falls in.
pub fn classify_lenient_plan_score(score: f64) -> String {
    if score < 63.9 {
        return "critical".to_string();
    }
    if score < 68.1 {
        return "high".to_string();
    }
    "low".to_string()
}
