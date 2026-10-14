use super::*;

use acme_kit::retry;

pub fn ttl_policy_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_group() -> i64 {
    let policy = retry::default_policy("ttl-policy-group");
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

/// Puts a ttl policy group code in its canonical TPG- form.
pub fn normalize_ttl_policy_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPG-") {
        return clean.clone();
    }
    format!("TPG-{}", clean.replace(" ", "-"))
}
