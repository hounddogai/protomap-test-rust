use super::*;

use acme_kit::retry;

/// Puts a isolation entry code in its canonical IE- form.
pub fn normalize_isolation_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IE-") {
        return clean.clone();
    }
    format!("IE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_entry() -> i64 {
    let policy = retry::default_policy("isolation-entry");
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
