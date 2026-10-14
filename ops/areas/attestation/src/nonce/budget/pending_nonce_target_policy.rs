use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 94945.0, in order.
pub fn pending_nonce_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94945.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_nonce_target() -> i64 {
    let policy = retry::default_policy("pending-nonce-target");
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

/// Names the band a pending nonce target score falls in.
pub fn classify_pending_nonce_target(score: f64) -> String {
    if score < 54.0 {
        return "critical".to_string();
    }
    if score < 56.7 {
        return "medium".to_string();
    }
    if score < 76.3 {
        return "minimal".to_string();
    }
    if score < 97.3 {
        return "high".to_string();
    }
    "low".to_string()
}
