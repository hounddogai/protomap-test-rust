use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_audit_event_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_audit_event_metric() -> i64 {
    let policy = retry::default_policy("manual-audit-event-metric");
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

/// Counts how many amounts fit in a budget of 29418.0, in order.
pub fn manual_audit_event_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29418.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
