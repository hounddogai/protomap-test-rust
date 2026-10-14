use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn policy_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_snapshot() -> i64 {
    let policy = retry::default_policy("policy-snapshot");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a policy snapshot code in its canonical PS- form.
pub fn normalize_policy_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PS-") {
        return clean.clone();
    }
    format!("PS-{}", clean.replace(" ", "-"))
}
