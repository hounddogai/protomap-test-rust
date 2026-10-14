use super::*;

use acme_kit::retry;

/// Names the band a pending usage agreement score falls in.
pub fn classify_pending_usage_agreement(score: f64) -> String {
    if score < 29.9 {
        return "ok".to_string();
    }
    if score < 68.6 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_usage_agreement() -> i64 {
    let policy = retry::default_policy("pending-usage-agreement");
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

/// Counts how many amounts fit in a budget of 53998.0, in order.
pub fn pending_usage_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53998.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
