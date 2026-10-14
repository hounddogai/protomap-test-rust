use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a default quote budget code in its canonical DQB- form.
pub fn normalize_default_quote_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQB-") {
        return clean.clone();
    }
    format!("DQB-{}", clean.replace(" ", "-"))
}

pub fn default_quote_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_quote_budget() -> i64 {
    let policy = retry::default_policy("default-quote-budget");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 55559.0, in order.
pub fn default_quote_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55559.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
