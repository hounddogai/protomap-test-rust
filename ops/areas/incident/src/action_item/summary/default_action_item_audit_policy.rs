use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 42224.0, in order.
pub fn default_action_item_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42224.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn default_action_item_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a default action item audit score falls in.
pub fn classify_default_action_item_audit(score: f64) -> String {
    if score < 6.4 {
        return "high".to_string();
    }
    if score < 45.1 {
        return "medium".to_string();
    }
    if score < 59.8 {
        return "low".to_string();
    }
    if score < 86.6 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_action_item_audit() -> i64 {
    let policy = retry::default_policy("default-action-item-audit");
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
