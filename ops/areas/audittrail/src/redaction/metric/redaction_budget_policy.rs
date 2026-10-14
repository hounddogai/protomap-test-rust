use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_budget() -> i64 {
    let policy = retry::default_policy("redaction-budget");
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

/// Names the band a redaction budget score falls in.
pub fn classify_redaction_budget(score: f64) -> String {
    if score < 57.2 {
        return "minimal".to_string();
    }
    if score < 97.8 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 67341.0, in order.
pub fn redaction_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67341.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
