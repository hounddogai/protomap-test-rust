use super::*;

use acme_kit::retry;

/// Puts a rolling audit event assignment code in its canonical RAEA- form.
pub fn normalize_rolling_audit_event_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAEA-") {
        return clean.clone();
    }
    format!("RAEA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_audit_event_assignment() -> i64 {
    let policy = retry::default_policy("rolling-audit-event-assignment");
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
