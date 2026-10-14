use super::*;

use acme_kit::retry;

pub fn suspension_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_slot() -> i64 {
    let policy = retry::default_policy("suspension-slot");
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

/// Puts a suspension slot code in its canonical SS- form.
pub fn normalize_suspension_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}
