use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_target() -> i64 {
    let policy = retry::default_policy("evidence-target");
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

/// Puts a evidence target code in its canonical ET- form.
pub fn normalize_evidence_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ET-") {
        return clean.clone();
    }
    format!("ET-{}", clean.replace(" ", "-"))
}
