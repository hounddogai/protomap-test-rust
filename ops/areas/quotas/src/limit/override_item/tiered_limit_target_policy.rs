use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_limit_target() -> i64 {
    let policy = retry::default_policy("tiered-limit-target");
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

/// Puts a tiered limit target code in its canonical TLT- form.
pub fn normalize_tiered_limit_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TLT-") {
        return clean.clone();
    }
    format!("TLT-{}", clean.replace(" ", "-"))
}
