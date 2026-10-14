use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a forecast snapshot score falls in.
pub fn classify_forecast_snapshot(score: f64) -> String {
    if score < 10.2 {
        return "watch".to_string();
    }
    if score < 14.8 {
        return "high".to_string();
    }
    if score < 16.3 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn merge_forecast_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn forecast_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_snapshot() -> i64 {
    let policy = retry::default_policy("forecast-snapshot");
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
