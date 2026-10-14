use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a federation audit score falls in.
pub fn classify_federation_audit(score: f64) -> String {
    if score < 5.6 {
        return "low".to_string();
    }
    if score < 56.5 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_federation_audit() -> i64 {
    let policy = retry::default_policy("federation-audit");
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

pub fn federation_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 25097.0, in order.
pub fn federation_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25097.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
