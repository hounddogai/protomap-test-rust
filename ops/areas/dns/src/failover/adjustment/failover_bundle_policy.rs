use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn failover_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_failover_bundle() -> i64 {
    let policy = retry::default_policy("failover-bundle");
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

/// Puts a failover bundle code in its canonical FB- form.
pub fn normalize_failover_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FB-") {
        return clean.clone();
    }
    format!("FB-{}", clean.replace(" ", "-"))
}
