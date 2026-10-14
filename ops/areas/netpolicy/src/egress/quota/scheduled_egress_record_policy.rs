use super::*;

use acme_kit::retry;

/// Puts a scheduled egress record code in its canonical SER- form.
pub fn normalize_scheduled_egress_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SER-") {
        return clean.clone();
    }
    format!("SER-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_egress_record() -> i64 {
    let policy = retry::default_policy("scheduled-egress-record");
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
