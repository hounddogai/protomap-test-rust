use super::*;

use acme_kit::retry;

/// Puts a default silence metric code in its canonical DSM- form.
pub fn normalize_default_silence_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DSM-") {
        return clean.clone();
    }
    format!("DSM-{}", clean.replace(" ", "-"))
}

pub fn default_silence_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_silence_metric() -> i64 {
    let policy = retry::default_policy("default-silence-metric");
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
