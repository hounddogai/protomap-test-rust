use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a silence estimate score falls in.
pub fn classify_silence_estimate(score: f64) -> String {
    if score < 25.0 {
        return "elevated".to_string();
    }
    if score < 68.4 {
        return "watch".to_string();
    }
    if score < 82.2 {
        return "medium".to_string();
    }
    if score < 82.8 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn silence_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_estimate() -> i64 {
    let policy = retry::default_policy("silence-estimate");
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

pub fn merge_silence_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
