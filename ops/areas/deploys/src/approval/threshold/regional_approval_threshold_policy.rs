use super::*;

use acme_kit::retry;

/// Puts a regional approval threshold code in its canonical RAT- form.
pub fn normalize_regional_approval_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAT-") {
        return clean.clone();
    }
    format!("RAT-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_approval_threshold() -> i64 {
    let policy = retry::default_policy("regional-approval-threshold");
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

pub fn regional_approval_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
