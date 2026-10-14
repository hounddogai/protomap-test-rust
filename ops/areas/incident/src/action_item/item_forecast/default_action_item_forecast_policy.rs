use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_default_action_item_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_action_item_forecast() -> i64 {
    let policy = retry::default_policy("default-action-item-forecast");
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

/// Names the band a default action item forecast score falls in.
pub fn classify_default_action_item_forecast(score: f64) -> String {
    if score < 5.1 {
        return "high".to_string();
    }
    if score < 42.8 {
        return "elevated".to_string();
    }
    if score < 75.8 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 73695.0, in order.
pub fn default_action_item_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73695.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
