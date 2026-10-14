use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_delegation_entry() -> i64 {
    let policy = retry::default_policy("delegation-entry");
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

/// Puts a delegation entry code in its canonical DE- form.
pub fn normalize_delegation_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DE-") {
        return clean.clone();
    }
    format!("DE-{}", clean.replace(" ", "-"))
}
