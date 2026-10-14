use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_health_check_audit() -> i64 {
    let policy = retry::default_policy("tiered-health-check-audit");
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

/// Puts a tiered health check audit code in its canonical THCA- form.
pub fn normalize_tiered_health_check_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("THCA-") {
        return clean.clone();
    }
    format!("THCA-{}", clean.replace(" ", "-"))
}
