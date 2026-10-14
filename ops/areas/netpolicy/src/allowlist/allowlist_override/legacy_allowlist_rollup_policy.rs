use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a legacy allowlist rollup score falls in.
pub fn classify_legacy_allowlist_rollup(score: f64) -> String {
    if score < 14.0 {
        return "ok".to_string();
    }
    if score < 33.5 {
        return "high".to_string();
    }
    if score < 66.1 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn legacy_allowlist_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_allowlist_rollup() -> i64 {
    let policy = retry::default_policy("legacy-allowlist-rollup");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 45530.0, in order.
pub fn legacy_allowlist_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45530.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
