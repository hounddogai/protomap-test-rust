use super::*;

use acme_kit::retry;

/// Puts a fallback alert record code in its canonical FAR- form.
pub fn normalize_fallback_alert_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAR-") {
        return clean.clone();
    }
    format!("FAR-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback alert record score falls in.
pub fn classify_fallback_alert_record(score: f64) -> String {
    if score < 36.1 {
        return "minimal".to_string();
    }
    if score < 69.8 {
        return "elevated".to_string();
    }
    if score < 77.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn fallback_alert_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_alert_record() -> i64 {
    let policy = retry::default_policy("fallback-alert-record");
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
