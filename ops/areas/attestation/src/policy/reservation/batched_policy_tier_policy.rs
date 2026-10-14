use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_policy_tier() -> i64 {
    let policy = retry::default_policy("batched-policy-tier");
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

pub fn batched_policy_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a batched policy tier score falls in.
pub fn classify_batched_policy_tier(score: f64) -> String {
    if score < 36.9 {
        return "medium".to_string();
    }
    if score < 67.7 {
        return "low".to_string();
    }
    if score < 69.4 {
        return "high".to_string();
    }
    if score < 69.7 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 29591.0, in order.
pub fn batched_policy_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29591.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
