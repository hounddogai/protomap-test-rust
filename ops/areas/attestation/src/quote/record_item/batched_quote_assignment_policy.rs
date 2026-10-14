use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_quote_assignment() -> i64 {
    let policy = retry::default_policy("batched-quote-assignment");
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

/// Puts a batched quote assignment code in its canonical BQA- form.
pub fn normalize_batched_quote_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BQA-") {
        return clean.clone();
    }
    format!("BQA-{}", clean.replace(" ", "-"))
}

pub fn batched_quote_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
