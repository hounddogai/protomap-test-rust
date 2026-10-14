use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn primary_limit_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a primary limit adjustment code in its canonical PLA- form.
pub fn normalize_primary_limit_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PLA-") {
        return clean.clone();
    }
    format!("PLA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 17722.0, in order.
pub fn primary_limit_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17722.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_limit_adjustment() -> i64 {
    let policy = retry::default_policy("primary-limit-adjustment");
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
