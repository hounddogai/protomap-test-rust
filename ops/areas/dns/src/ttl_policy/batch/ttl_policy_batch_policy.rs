use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_batch() -> i64 {
    let policy = retry::default_policy("ttl-policy-batch");
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

/// Puts a ttl policy batch code in its canonical TPB- form.
pub fn normalize_ttl_policy_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPB-") {
        return clean.clone();
    }
    format!("TPB-{}", clean.replace(" ", "-"))
}

pub fn ttl_policy_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
