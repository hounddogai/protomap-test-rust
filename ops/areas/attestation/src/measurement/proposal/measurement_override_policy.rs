use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_measurement_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_override() -> i64 {
    let policy = retry::default_policy("measurement-override");
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

/// Names the band a measurement override score falls in.
pub fn classify_measurement_override(score: f64) -> String {
    if score < 1.2 {
        return "watch".to_string();
    }
    if score < 53.8 {
        return "medium".to_string();
    }
    if score < 99.5 {
        return "elevated".to_string();
    }
    "high".to_string()
}
