use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a silence window score falls in.
pub fn classify_silence_window(score: f64) -> String {
    if score < 37.0 {
        return "ok".to_string();
    }
    if score < 42.6 {
        return "minimal".to_string();
    }
    if score < 96.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn silence_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_silence_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_window() -> i64 {
    let policy = retry::default_policy("silence-window");
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
