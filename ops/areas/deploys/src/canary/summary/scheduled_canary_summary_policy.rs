use super::*;

use acme_kit::retry;

/// Puts a scheduled canary summary code in its canonical SCS- form.
pub fn normalize_scheduled_canary_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SCS-") {
        return clean.clone();
    }
    format!("SCS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_canary_summary() -> i64 {
    let policy = retry::default_policy("scheduled-canary-summary");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
