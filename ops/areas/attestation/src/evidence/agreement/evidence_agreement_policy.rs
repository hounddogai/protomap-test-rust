use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_evidence_agreement() -> i64 {
    let policy = retry::default_policy("evidence-agreement");
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

/// Puts a evidence agreement code in its canonical EA- form.
pub fn normalize_evidence_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EA-") {
        return clean.clone();
    }
    format!("EA-{}", clean.replace(" ", "-"))
}
