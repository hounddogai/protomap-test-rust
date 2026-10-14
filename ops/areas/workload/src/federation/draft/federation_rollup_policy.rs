use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_federation_rollup() -> i64 {
    let policy = retry::default_policy("federation-rollup");
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

/// Puts a federation rollup code in its canonical FR- form.
pub fn normalize_federation_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FR-") {
        return clean.clone();
    }
    format!("FR-{}", clean.replace(" ", "-"))
}
