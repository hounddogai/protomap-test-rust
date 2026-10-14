use super::*;

use acme_kit::retry;

/// Puts a pending policy schedule code in its canonical PPS- form.
pub fn normalize_pending_policy_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPS-") {
        return clean.clone();
    }
    format!("PPS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_policy_schedule() -> i64 {
    let policy = retry::default_policy("pending-policy-schedule");
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

pub fn pending_policy_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
