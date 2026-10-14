use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_federation_slot() -> i64 {
    let policy = retry::default_policy("federation-slot");
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

/// Puts a federation slot code in its canonical FS- form.
pub fn normalize_federation_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}
