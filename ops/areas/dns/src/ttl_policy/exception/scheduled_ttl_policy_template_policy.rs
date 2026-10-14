use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a scheduled ttl policy template score falls in.
pub fn classify_scheduled_ttl_policy_template(score: f64) -> String {
    if score < 29.7 {
        return "critical".to_string();
    }
    if score < 33.8 {
        return "minimal".to_string();
    }
    if score < 44.7 {
        return "ok".to_string();
    }
    if score < 48.4 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_ttl_policy_template() -> i64 {
    let policy = retry::default_policy("scheduled-ttl-policy-template");
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

pub fn merge_scheduled_ttl_policy_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
