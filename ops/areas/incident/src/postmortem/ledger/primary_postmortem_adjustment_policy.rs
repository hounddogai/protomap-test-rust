use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_postmortem_adjustment() -> i64 {
    let policy = retry::default_policy("primary-postmortem-adjustment");
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

pub fn merge_primary_postmortem_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn primary_postmortem_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a primary postmortem adjustment score falls in.
pub fn classify_primary_postmortem_adjustment(score: f64) -> String {
    if score < 29.9 {
        return "minimal".to_string();
    }
    if score < 73.4 {
        return "medium".to_string();
    }
    "ok".to_string()
}
