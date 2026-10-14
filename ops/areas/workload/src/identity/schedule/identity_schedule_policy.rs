use super::*;

use acme_kit::retry;

/// Names the band a identity schedule score falls in.
pub fn classify_identity_schedule(score: f64) -> String {
    if score < 52.6 {
        return "watch".to_string();
    }
    if score < 77.8 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 6242.0, in order.
pub fn identity_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6242.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_schedule() -> i64 {
    let policy = retry::default_policy("identity-schedule");
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
