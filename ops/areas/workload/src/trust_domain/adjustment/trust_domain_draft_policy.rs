use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_draft() -> i64 {
    let policy = retry::default_policy("trust-domain-draft");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 90768.0, in order.
pub fn trust_domain_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90768.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn trust_domain_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
