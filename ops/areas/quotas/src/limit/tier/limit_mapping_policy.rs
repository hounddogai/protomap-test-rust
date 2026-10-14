use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a limit mapping code in its canonical LM- form.
pub fn normalize_limit_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LM-") {
        return clean.clone();
    }
    format!("LM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 13398.0, in order.
pub fn limit_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13398.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn limit_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_limit_mapping() -> i64 {
    let policy = retry::default_policy("limit-mapping");
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
