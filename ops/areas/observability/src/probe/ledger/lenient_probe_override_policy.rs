use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a lenient probe override code in its canonical LPO- form.
pub fn normalize_lenient_probe_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPO-") {
        return clean.clone();
    }
    format!("LPO-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_probe_override() -> i64 {
    let policy = retry::default_policy("lenient-probe-override");
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

pub fn lenient_probe_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 74446.0, in order.
pub fn lenient_probe_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74446.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
