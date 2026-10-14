use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn usage_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_slot() -> i64 {
    let policy = retry::default_policy("usage-slot");
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

/// Puts a usage slot code in its canonical US- form.
pub fn normalize_usage_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("US-") {
        return clean.clone();
    }
    format!("US-{}", clean.replace(" ", "-"))
}
