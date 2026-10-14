use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn primary_isolation_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a primary isolation variance code in its canonical PIV- form.
pub fn normalize_primary_isolation_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PIV-") {
        return clean.clone();
    }
    format!("PIV-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_isolation_variance() -> i64 {
    let policy = retry::default_policy("primary-isolation-variance");
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
