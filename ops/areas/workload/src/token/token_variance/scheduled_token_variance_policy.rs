use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 75622.0, in order.
pub fn scheduled_token_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75622.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_token_variance() -> i64 {
    let policy = retry::default_policy("scheduled-token-variance");
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

pub fn scheduled_token_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a scheduled token variance score falls in.
pub fn classify_scheduled_token_variance(score: f64) -> String {
    if score < 27.0 {
        return "minimal".to_string();
    }
    if score < 66.6 {
        return "elevated".to_string();
    }
    "high".to_string()
}
