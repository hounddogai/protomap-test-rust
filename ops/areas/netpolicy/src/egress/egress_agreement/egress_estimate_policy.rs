use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a egress estimate code in its canonical EE- form.
pub fn normalize_egress_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EE-") {
        return clean.clone();
    }
    format!("EE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_estimate() -> i64 {
    let policy = retry::default_policy("egress-estimate");
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

pub fn egress_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
