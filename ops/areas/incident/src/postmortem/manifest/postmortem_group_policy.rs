use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_group() -> i64 {
    let policy = retry::default_policy("postmortem-group");
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

pub fn merge_postmortem_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a postmortem group score falls in.
pub fn classify_postmortem_group(score: f64) -> String {
    if score < 25.2 {
        return "low".to_string();
    }
    if score < 27.1 {
        return "watch".to_string();
    }
    if score < 53.6 {
        return "ok".to_string();
    }
    if score < 73.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn postmortem_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
