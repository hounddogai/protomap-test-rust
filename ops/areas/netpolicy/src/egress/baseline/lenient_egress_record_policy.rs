use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_egress_record() -> i64 {
    let policy = retry::default_policy("lenient-egress-record");
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

/// Puts a lenient egress record code in its canonical LER- form.
pub fn normalize_lenient_egress_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LER-") {
        return clean.clone();
    }
    format!("LER-{}", clean.replace(" ", "-"))
}
