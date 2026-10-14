use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_action_item_preference() -> i64 {
    let policy = retry::default_policy("regional-action-item-preference");
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

/// Names the band a regional action item preference score falls in.
pub fn classify_regional_action_item_preference(score: f64) -> String {
    if score < 18.3 {
        return "elevated".to_string();
    }
    if score < 51.9 {
        return "ok".to_string();
    }
    if score < 89.8 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn merge_regional_action_item_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
