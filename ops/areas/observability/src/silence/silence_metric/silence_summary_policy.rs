use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_summary() -> i64 {
    let policy = retry::default_policy("silence-summary");
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

pub fn silence_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a silence summary code in its canonical SS- form.
pub fn normalize_silence_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 37944.0, in order.
pub fn silence_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37944.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
