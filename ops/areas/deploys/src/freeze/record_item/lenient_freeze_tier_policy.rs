use super::*;

use acme_kit::retry;

/// Puts a lenient freeze tier code in its canonical LFT- form.
pub fn normalize_lenient_freeze_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFT-") {
        return clean.clone();
    }
    format!("LFT-{}", clean.replace(" ", "-"))
}

pub fn lenient_freeze_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_freeze_tier() -> i64 {
    let policy = retry::default_policy("lenient-freeze-tier");
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
