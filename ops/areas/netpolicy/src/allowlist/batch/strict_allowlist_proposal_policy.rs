use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_allowlist_proposal() -> i64 {
    let policy = retry::default_policy("strict-allowlist-proposal");
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

/// Puts a strict allowlist proposal code in its canonical SAP- form.
pub fn normalize_strict_allowlist_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAP-") {
        return clean.clone();
    }
    format!("SAP-{}", clean.replace(" ", "-"))
}
