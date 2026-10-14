use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn postmortem_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a postmortem review code in its canonical PR- form.
pub fn normalize_postmortem_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}

pub fn postmortem_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_review() -> i64 {
    let policy = retry::default_policy("postmortem-review");
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
