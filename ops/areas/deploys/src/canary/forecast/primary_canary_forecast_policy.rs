use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_canary_forecast() -> i64 {
    let policy = retry::default_policy("primary-canary-forecast");
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

pub fn merge_primary_canary_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a primary canary forecast score falls in.
pub fn classify_primary_canary_forecast(score: f64) -> String {
    if score < 54.0 {
        return "watch".to_string();
    }
    if score < 70.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a primary canary forecast code in its canonical PCF- form.
pub fn normalize_primary_canary_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PCF-") {
        return clean.clone();
    }
    format!("PCF-{}", clean.replace(" ", "-"))
}
