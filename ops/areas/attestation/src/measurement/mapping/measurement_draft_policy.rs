use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_draft() -> i64 {
    let policy = retry::default_policy("measurement-draft");
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

/// Names the band a measurement draft score falls in.
pub fn classify_measurement_draft(score: f64) -> String {
    if score < 45.8 {
        return "low".to_string();
    }
    if score < 57.7 {
        return "high".to_string();
    }
    if score < 75.0 {
        return "critical".to_string();
    }
    if score < 89.1 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn merge_measurement_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn measurement_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
