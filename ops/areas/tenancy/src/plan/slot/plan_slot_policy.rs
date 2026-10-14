use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_plan_slot() -> i64 {
    let policy = retry::default_policy("plan-slot");
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

/// Counts how many amounts fit in a budget of 60454.0, in order.
pub fn plan_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60454.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a plan slot score falls in.
pub fn classify_plan_slot(score: f64) -> String {
    if score < 17.7 {
        return "low".to_string();
    }
    if score < 41.7 {
        return "watch".to_string();
    }
    if score < 44.1 {
        return "elevated".to_string();
    }
    if score < 60.4 {
        return "critical".to_string();
    }
    "high".to_string()
}
