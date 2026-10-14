use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_allowlist_checkpoint() -> i64 {
    let policy = retry::default_policy("batched-allowlist-checkpoint");
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

pub fn batched_allowlist_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched allowlist checkpoint score falls in.
pub fn classify_batched_allowlist_checkpoint(score: f64) -> String {
    if score < 31.0 {
        return "minimal".to_string();
    }
    if score < 64.1 {
        return "ok".to_string();
    }
    if score < 92.5 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 33236.0, in order.
pub fn batched_allowlist_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33236.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
