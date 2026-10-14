use super::*;

use acme_kit::retry;

/// Puts a adaptive suspension quota code in its canonical ASQ- form.
pub fn normalize_adaptive_suspension_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ASQ-") {
        return clean.clone();
    }
    format!("ASQ-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_suspension_quota() -> i64 {
    let policy = retry::default_policy("adaptive-suspension-quota");
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
