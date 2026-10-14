use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_delegation_draft() -> i64 {
    let policy = retry::default_policy("global-delegation-draft");
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

/// Puts a global delegation draft code in its canonical GDD- form.
pub fn normalize_global_delegation_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GDD-") {
        return clean.clone();
    }
    format!("GDD-{}", clean.replace(" ", "-"))
}
