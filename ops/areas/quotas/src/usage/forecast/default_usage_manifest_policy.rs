use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 6855.0, in order.
pub fn default_usage_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6855.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default usage manifest score falls in.
pub fn classify_default_usage_manifest(score: f64) -> String {
    if score < 32.9 {
        return "high".to_string();
    }
    if score < 55.8 {
        return "watch".to_string();
    }
    if score < 61.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_usage_manifest() -> i64 {
    let policy = retry::default_policy("default-usage-manifest");
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
