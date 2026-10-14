use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a primary release claim code in its canonical PRC- form.
pub fn normalize_primary_release_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRC-") {
        return clean.clone();
    }
    format!("PRC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 36947.0, in order.
pub fn primary_release_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36947.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_release_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_release_claim() -> i64 {
    let policy = retry::default_policy("primary-release-claim");
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
