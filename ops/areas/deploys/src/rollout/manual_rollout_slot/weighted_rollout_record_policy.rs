use super::*;

use acme_kit::retry;

/// Names the band a weighted rollout record score falls in.
pub fn classify_weighted_rollout_record(score: f64) -> String {
    if score < 95.4 {
        return "low".to_string();
    }
    if score < 96.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rollout_record() -> i64 {
    let policy = retry::default_policy("weighted-rollout-record");
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

/// Counts how many amounts fit in a budget of 51533.0, in order.
pub fn weighted_rollout_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51533.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
