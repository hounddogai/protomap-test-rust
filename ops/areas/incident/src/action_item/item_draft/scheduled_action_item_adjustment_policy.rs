use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_scheduled_action_item_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled action item adjustment score falls in.
pub fn classify_scheduled_action_item_adjustment(score: f64) -> String {
    if score < 11.4 {
        return "watch".to_string();
    }
    if score < 37.3 {
        return "low".to_string();
    }
    if score < 48.8 {
        return "critical".to_string();
    }
    if score < 60.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn scheduled_action_item_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_action_item_adjustment() -> i64 {
    let policy = retry::default_policy("scheduled-action-item-adjustment");
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
