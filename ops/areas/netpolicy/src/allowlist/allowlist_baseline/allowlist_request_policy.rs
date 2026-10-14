use super::*;

use acme_kit::retry;

/// Puts a allowlist request code in its canonical AR- form.
pub fn normalize_allowlist_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AR-") {
        return clean.clone();
    }
    format!("AR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_request() -> i64 {
    let policy = retry::default_policy("allowlist-request");
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
