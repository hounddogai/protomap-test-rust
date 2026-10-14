use super::*;

use acme_kit::retry;

/// Puts a token digest code in its canonical TD- form.
pub fn normalize_token_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TD-") {
        return clean.clone();
    }
    format!("TD-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_token_digest() -> i64 {
    let policy = retry::default_policy("token-digest");
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
