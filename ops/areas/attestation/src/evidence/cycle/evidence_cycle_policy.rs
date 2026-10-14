use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 414.0, in order.
pub fn evidence_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 414.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn evidence_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_cycle() -> i64 {
    let policy = retry::default_policy("evidence-cycle");
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
