use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn default_budget_burn_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a default budget burn snapshot score falls in.
pub fn classify_default_budget_burn_snapshot(score: f64) -> String {
    if score < 28.1 {
        return "low".to_string();
    }
    if score < 28.3 {
        return "critical".to_string();
    }
    if score < 74.8 {
        return "medium".to_string();
    }
    if score < 94.0 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_budget_burn_snapshot() -> i64 {
    let policy = retry::default_policy("default-budget-burn-snapshot");
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
