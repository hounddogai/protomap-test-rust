use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a zone request code in its canonical ZR- form.
pub fn normalize_zone_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZR-") {
        return clean.clone();
    }
    format!("ZR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_request() -> i64 {
    let policy = retry::default_policy("zone-request");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn zone_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
