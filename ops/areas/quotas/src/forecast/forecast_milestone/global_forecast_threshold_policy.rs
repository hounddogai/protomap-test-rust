use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_forecast_threshold() -> i64 {
    let policy = retry::default_policy("global-forecast-threshold");
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

pub fn merge_global_forecast_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a global forecast threshold score falls in.
pub fn classify_global_forecast_threshold(score: f64) -> String {
    if score < 3.3 {
        return "critical".to_string();
    }
    if score < 57.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
