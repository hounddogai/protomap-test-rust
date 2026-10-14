use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_allocation() -> i64 {
    let policy = retry::default_policy("flow-log-allocation");
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

/// Puts a flow log allocation code in its canonical FLA- form.
pub fn normalize_flow_log_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLA-") {
        return clean.clone();
    }
    format!("FLA-{}", clean.replace(" ", "-"))
}
