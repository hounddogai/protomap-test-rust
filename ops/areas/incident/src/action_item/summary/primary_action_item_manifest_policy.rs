use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_primary_action_item_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_action_item_manifest() -> i64 {
    let policy = retry::default_policy("primary-action-item-manifest");
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

/// Names the band a primary action item manifest score falls in.
pub fn classify_primary_action_item_manifest(score: f64) -> String {
    if score < 48.2 {
        return "minimal".to_string();
    }
    if score < 75.0 {
        return "low".to_string();
    }
    if score < 96.4 {
        return "high".to_string();
    }
    "critical".to_string()
}
