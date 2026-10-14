use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a scheduled usage estimate score falls in.
pub fn classify_scheduled_usage_estimate(score: f64) -> String {
    if score < 41.5 {
        return "high".to_string();
    }
    if score < 52.0 {
        return "low".to_string();
    }
    if score < 66.7 {
        return "ok".to_string();
    }
    if score < 73.8 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_usage_estimate() -> i64 {
    let policy = retry::default_policy("scheduled-usage-estimate");
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

pub fn scheduled_usage_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 97756.0, in order.
pub fn scheduled_usage_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97756.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
