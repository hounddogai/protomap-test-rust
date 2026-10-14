use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a legacy retention estimate score falls in.
pub fn classify_legacy_retention_estimate(score: f64) -> String {
    if score < 2.1 {
        return "ok".to_string();
    }
    if score < 69.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 46035.0, in order.
pub fn legacy_retention_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46035.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_retention_estimate() -> i64 {
    let policy = retry::default_policy("legacy-retention-estimate");
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

pub fn legacy_retention_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
