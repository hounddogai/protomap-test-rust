use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_evidence_quota() -> i64 {
    let policy = retry::default_policy("seasonal-evidence-quota");
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

/// Puts a seasonal evidence quota code in its canonical SEQ- form.
pub fn normalize_seasonal_evidence_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEQ-") {
        return clean.clone();
    }
    format!("SEQ-{}", clean.replace(" ", "-"))
}

pub fn seasonal_evidence_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
