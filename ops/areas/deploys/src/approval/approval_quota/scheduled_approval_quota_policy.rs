use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_approval_quota() -> i64 {
    let policy = retry::default_policy("scheduled-approval-quota");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a scheduled approval quota code in its canonical SAQ- form.
pub fn normalize_scheduled_approval_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAQ-") {
        return clean.clone();
    }
    format!("SAQ-{}", clean.replace(" ", "-"))
}

pub fn scheduled_approval_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a scheduled approval quota score falls in.
pub fn classify_scheduled_approval_quota(score: f64) -> String {
    if score < 69.3 {
        return "ok".to_string();
    }
    if score < 73.3 {
        return "low".to_string();
    }
    "minimal".to_string()
}
