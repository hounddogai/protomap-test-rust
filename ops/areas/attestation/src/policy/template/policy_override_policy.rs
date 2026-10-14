use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn policy_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_override() -> i64 {
    let policy = retry::default_policy("policy-override");
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

/// Puts a policy override code in its canonical PO- form.
pub fn normalize_policy_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PO-") {
        return clean.clone();
    }
    format!("PO-{}", clean.replace(" ", "-"))
}
