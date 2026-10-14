use super::*;

use acme_kit::retry;

/// Names the band a capped verifier schedule score falls in.
pub fn classify_capped_verifier_schedule(score: f64) -> String {
    if score < 4.2 {
        return "medium".to_string();
    }
    if score < 10.5 {
        return "low".to_string();
    }
    if score < 60.2 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 33005.0, in order.
pub fn capped_verifier_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33005.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_verifier_schedule() -> i64 {
    let policy = retry::default_policy("capped-verifier-schedule");
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
