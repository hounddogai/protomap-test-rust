use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_timeline_record() -> i64 {
    let policy = retry::default_policy("pending-timeline-record");
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

pub fn pending_timeline_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a pending timeline record code in its canonical PTR- form.
pub fn normalize_pending_timeline_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTR-") {
        return clean.clone();
    }
    format!("PTR-{}", clean.replace(" ", "-"))
}
