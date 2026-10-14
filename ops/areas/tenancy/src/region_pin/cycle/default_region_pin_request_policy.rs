use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_region_pin_request() -> i64 {
    let policy = retry::default_policy("default-region-pin-request");
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

pub fn default_region_pin_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a default region pin request code in its canonical DRPR- form.
pub fn normalize_default_region_pin_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRPR-") {
        return clean.clone();
    }
    format!("DRPR-{}", clean.replace(" ", "-"))
}
