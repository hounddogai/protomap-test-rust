use super::*;

use acme_kit::retry;

/// Puts a alert quota code in its canonical AQ- form.
pub fn normalize_alert_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AQ-") {
        return clean.clone();
    }
    format!("AQ-{}", clean.replace(" ", "-"))
}

pub fn alert_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_quota() -> i64 {
    let policy = retry::default_policy("alert-quota");
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
