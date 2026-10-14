use super::*;

use acme_kit::retry;

/// Puts a limit checkpoint code in its canonical LC- form.
pub fn normalize_limit_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LC-") {
        return clean.clone();
    }
    format!("LC-{}", clean.replace(" ", "-"))
}

pub fn limit_checkpoint_span(start: i64, end: i64) -> i64 {
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
pub fn probe_limit_checkpoint() -> i64 {
    let policy = retry::default_policy("limit-checkpoint");
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
