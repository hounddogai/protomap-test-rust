use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn isolation_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a isolation claim code in its canonical IC- form.
pub fn normalize_isolation_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IC-") {
        return clean.clone();
    }
    format!("IC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 7851.0, in order.
pub fn isolation_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7851.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_claim() -> i64 {
    let policy = retry::default_policy("isolation-claim");
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
