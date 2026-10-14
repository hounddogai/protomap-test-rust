use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a weighted rule variance code in its canonical WRV- form.
pub fn normalize_weighted_rule_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRV-") {
        return clean.clone();
    }
    format!("WRV-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rule_variance() -> i64 {
    let policy = retry::default_policy("weighted-rule-variance");
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

pub fn weighted_rule_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 10677.0, in order.
pub fn weighted_rule_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10677.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
