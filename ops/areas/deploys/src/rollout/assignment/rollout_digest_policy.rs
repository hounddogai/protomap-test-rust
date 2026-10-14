use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rollout_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a rollout digest code in its canonical RD- form.
pub fn normalize_rollout_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RD-") {
        return clean.clone();
    }
    format!("RD-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollout_digest() -> i64 {
    let policy = retry::default_policy("rollout-digest");
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

/// Counts how many amounts fit in a budget of 59775.0, in order.
pub fn rollout_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59775.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
