use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a escalation snapshot score falls in.
pub fn classify_escalation_snapshot(score: f64) -> String {
    if score < 1.3 {
        return "critical".to_string();
    }
    if score < 12.9 {
        return "medium".to_string();
    }
    if score < 47.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn escalation_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 97626.0, in order.
pub fn escalation_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97626.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_snapshot() -> i64 {
    let policy = retry::default_policy("escalation-snapshot");
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
