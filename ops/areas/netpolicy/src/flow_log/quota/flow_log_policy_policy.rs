use super::*;

use acme_kit::retry;

/// Puts a flow log policy code in its canonical FLP- form.
pub fn normalize_flow_log_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLP-") {
        return clean.clone();
    }
    format!("FLP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_policy() -> i64 {
    let policy = retry::default_policy("flow-log-policy");
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
