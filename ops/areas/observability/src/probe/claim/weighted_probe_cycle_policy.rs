use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_probe_cycle() -> i64 {
    let policy = retry::default_policy("weighted-probe-cycle");
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

/// Names the band a weighted probe cycle score falls in.
pub fn classify_weighted_probe_cycle(score: f64) -> String {
    if score < 17.9 {
        return "medium".to_string();
    }
    if score < 53.8 {
        return "high".to_string();
    }
    if score < 85.1 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 49353.0, in order.
pub fn weighted_probe_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49353.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
