use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_template() -> i64 {
    let policy = retry::default_policy("overage-template");
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

/// Puts a overage template code in its canonical OT- form.
pub fn normalize_overage_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OT-") {
        return clean.clone();
    }
    format!("OT-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 95178.0, in order.
pub fn overage_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95178.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn overage_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
