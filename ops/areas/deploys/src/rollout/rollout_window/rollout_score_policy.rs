use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a rollout score score falls in.
pub fn classify_rollout_score(score: f64) -> String {
    if score < 1.4 {
        return "watch".to_string();
    }
    if score < 94.2 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollout_score() -> i64 {
    let policy = retry::default_policy("rollout-score");
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

/// Counts how many amounts fit in a budget of 60397.0, in order.
pub fn rollout_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60397.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rollout_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
