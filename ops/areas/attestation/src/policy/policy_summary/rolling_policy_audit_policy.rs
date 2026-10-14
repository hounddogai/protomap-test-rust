use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 23099.0, in order.
pub fn rolling_policy_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23099.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_policy_audit() -> i64 {
    let policy = retry::default_policy("rolling-policy-audit");
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

/// Puts a rolling policy audit code in its canonical RPA- form.
pub fn normalize_rolling_policy_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPA-") {
        return clean.clone();
    }
    format!("RPA-{}", clean.replace(" ", "-"))
}

pub fn rolling_policy_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
