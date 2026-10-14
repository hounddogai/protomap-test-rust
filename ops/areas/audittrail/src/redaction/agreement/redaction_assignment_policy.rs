use super::*;

use acme_kit::retry;

/// Puts a redaction assignment code in its canonical RA- form.
pub fn normalize_redaction_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_assignment() -> i64 {
    let policy = retry::default_policy("redaction-assignment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
