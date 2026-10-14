use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_alert_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn lenient_alert_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a lenient alert profile code in its canonical LAP- form.
pub fn normalize_lenient_alert_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAP-") {
        return clean.clone();
    }
    format!("LAP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_alert_profile() -> i64 {
    let policy = retry::default_policy("lenient-alert-profile");
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
