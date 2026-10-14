use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_token_forecast() -> i64 {
    let policy = retry::default_policy("batched-token-forecast");
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

pub fn batched_token_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_batched_token_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched token forecast score falls in.
pub fn classify_batched_token_forecast(score: f64) -> String {
    if score < 1.1 {
        return "high".to_string();
    }
    if score < 53.8 {
        return "medium".to_string();
    }
    if score < 63.2 {
        return "low".to_string();
    }
    "ok".to_string()
}
