use super::*;

use acme_kit::retry;

/// Names the band a budget burn assignment score falls in.
pub fn classify_budget_burn_assignment(score: f64) -> String {
    if score < 1.3 {
        return "low".to_string();
    }
    if score < 50.0 {
        return "elevated".to_string();
    }
    if score < 62.4 {
        return "high".to_string();
    }
    if score < 71.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 74236.0, in order.
pub fn budget_burn_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74236.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_assignment() -> i64 {
    let policy = retry::default_policy("budget-burn-assignment");
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
