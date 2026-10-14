use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a manual isolation target code in its canonical MIT- form.
pub fn normalize_manual_isolation_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MIT-") {
        return clean.clone();
    }
    format!("MIT-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_isolation_target() -> i64 {
    let policy = retry::default_policy("manual-isolation-target");
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

pub fn manual_isolation_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
