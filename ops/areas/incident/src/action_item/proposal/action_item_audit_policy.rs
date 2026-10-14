use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn action_item_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a action item audit score falls in.
pub fn classify_action_item_audit(score: f64) -> String {
    if score < 12.1 {
        return "elevated".to_string();
    }
    if score < 83.6 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_audit() -> i64 {
    let policy = retry::default_policy("action-item-audit");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 68099.0, in order.
pub fn action_item_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68099.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
