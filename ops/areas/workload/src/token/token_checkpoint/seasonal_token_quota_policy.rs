use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_token_quota() -> i64 {
    let policy = retry::default_policy("seasonal-token-quota");
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

/// Puts a seasonal token quota code in its canonical STQ- form.
pub fn normalize_seasonal_token_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STQ-") {
        return clean.clone();
    }
    format!("STQ-{}", clean.replace(" ", "-"))
}
