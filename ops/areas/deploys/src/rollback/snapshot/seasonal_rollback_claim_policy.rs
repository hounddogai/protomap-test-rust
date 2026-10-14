use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a seasonal rollback claim code in its canonical SRC- form.
pub fn normalize_seasonal_rollback_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRC-") {
        return clean.clone();
    }
    format!("SRC-{}", clean.replace(" ", "-"))
}

pub fn seasonal_rollback_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_rollback_claim() -> i64 {
    let policy = retry::default_policy("seasonal-rollback-claim");
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
