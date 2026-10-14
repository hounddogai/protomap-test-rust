use super::*;

use acme_kit::retry;

pub fn pending_slo_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a pending slo profile code in its canonical PSP- form.
pub fn normalize_pending_slo_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSP-") {
        return clean.clone();
    }
    format!("PSP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_slo_profile() -> i64 {
    let policy = retry::default_policy("pending-slo-profile");
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
