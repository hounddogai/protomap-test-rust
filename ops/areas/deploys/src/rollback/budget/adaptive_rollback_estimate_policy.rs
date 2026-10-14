use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a adaptive rollback estimate code in its canonical ARE- form.
pub fn normalize_adaptive_rollback_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARE-") {
        return clean.clone();
    }
    format!("ARE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_rollback_estimate() -> i64 {
    let policy = retry::default_policy("adaptive-rollback-estimate");
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

pub fn adaptive_rollback_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
