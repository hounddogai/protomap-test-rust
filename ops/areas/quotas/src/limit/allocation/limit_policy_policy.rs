use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a limit policy code in its canonical LP- form.
pub fn normalize_limit_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LP-") {
        return clean.clone();
    }
    format!("LP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_limit_policy() -> i64 {
    let policy = retry::default_policy("limit-policy");
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

/// Counts how many amounts fit in a budget of 55717.0, in order.
pub fn limit_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55717.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn limit_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
