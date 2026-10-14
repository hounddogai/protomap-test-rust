use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 2619.0, in order.
pub fn primary_measurement_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2619.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_measurement_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a primary measurement rule code in its canonical PMR- form.
pub fn normalize_primary_measurement_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PMR-") {
        return clean.clone();
    }
    format!("PMR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_measurement_rule() -> i64 {
    let policy = retry::default_policy("primary-measurement-rule");
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
