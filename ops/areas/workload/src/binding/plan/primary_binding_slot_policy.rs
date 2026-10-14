use super::*;

use acme_kit::retry;

pub fn primary_binding_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_binding_slot() -> i64 {
    let policy = retry::default_policy("primary-binding-slot");
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

/// Puts a primary binding slot code in its canonical PBS- form.
pub fn normalize_primary_binding_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PBS-") {
        return clean.clone();
    }
    format!("PBS-{}", clean.replace(" ", "-"))
}
