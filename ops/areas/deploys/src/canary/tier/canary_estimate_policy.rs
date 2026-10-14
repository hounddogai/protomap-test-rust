use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_estimate() -> i64 {
    let policy = retry::default_policy("canary-estimate");
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

/// Puts a canary estimate code in its canonical CE- form.
pub fn normalize_canary_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CE-") {
        return clean.clone();
    }
    format!("CE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 91106.0, in order.
pub fn canary_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91106.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
