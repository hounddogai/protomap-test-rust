use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 13321.0, in order.
pub fn strict_release_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13321.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_release_forecast() -> i64 {
    let policy = retry::default_policy("strict-release-forecast");
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

/// Names the band a strict release forecast score falls in.
pub fn classify_strict_release_forecast(score: f64) -> String {
    if score < 15.5 {
        return "low".to_string();
    }
    if score < 54.4 {
        return "elevated".to_string();
    }
    if score < 59.2 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn merge_strict_release_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
