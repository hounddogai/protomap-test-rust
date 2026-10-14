use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn scheduled_silence_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_silence_budget() -> i64 {
    let policy = retry::default_policy("scheduled-silence-budget");
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

/// Puts a scheduled silence budget code in its canonical SSB- form.
pub fn normalize_scheduled_silence_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSB-") {
        return clean.clone();
    }
    format!("SSB-{}", clean.replace(" ", "-"))
}
