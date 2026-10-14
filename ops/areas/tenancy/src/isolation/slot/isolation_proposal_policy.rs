use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_proposal() -> i64 {
    let policy = retry::default_policy("isolation-proposal");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a isolation proposal code in its canonical IP- form.
pub fn normalize_isolation_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IP-") {
        return clean.clone();
    }
    format!("IP-{}", clean.replace(" ", "-"))
}
