use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 94990.0, in order.
pub fn tiered_canary_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94990.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tiered canary score score falls in.
pub fn classify_tiered_canary_score(score: f64) -> String {
    if score < 7.8 {
        return "medium".to_string();
    }
    if score < 29.3 {
        return "elevated".to_string();
    }
    if score < 37.6 {
        return "critical".to_string();
    }
    if score < 48.2 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_canary_score() -> i64 {
    let policy = retry::default_policy("tiered-canary-score");
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

pub fn tiered_canary_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
