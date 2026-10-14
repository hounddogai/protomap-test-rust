use super::*;

use acme_kit::retry;

/// Puts a runbook milestone code in its canonical RM- form.
pub fn normalize_runbook_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RM-") {
        return clean.clone();
    }
    format!("RM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_milestone() -> i64 {
    let policy = retry::default_policy("runbook-milestone");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
