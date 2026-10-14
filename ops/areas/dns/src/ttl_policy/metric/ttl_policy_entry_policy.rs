use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn ttl_policy_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn ttl_policy_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_entry() -> i64 {
    let policy = retry::default_policy("ttl-policy-entry");
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

/// Puts a ttl policy entry code in its canonical TPE- form.
pub fn normalize_ttl_policy_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPE-") {
        return clean.clone();
    }
    format!("TPE-{}", clean.replace(" ", "-"))
}
