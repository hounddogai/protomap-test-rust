use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 98840.0, in order.
pub fn verifier_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98840.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_verifier_override() -> i64 {
    let policy = retry::default_policy("verifier-override");
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

/// Puts a verifier override code in its canonical VO- form.
pub fn normalize_verifier_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VO-") {
        return clean.clone();
    }
    format!("VO-{}", clean.replace(" ", "-"))
}

pub fn verifier_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
