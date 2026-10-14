use super::*;

use acme_kit::retry;

/// Puts a batched silence segment code in its canonical BSS- form.
pub fn normalize_batched_silence_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BSS-") {
        return clean.clone();
    }
    format!("BSS-{}", clean.replace(" ", "-"))
}

pub fn batched_silence_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_silence_segment() -> i64 {
    let policy = retry::default_policy("batched-silence-segment");
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

/// Names the band a batched silence segment score falls in.
pub fn classify_batched_silence_segment(score: f64) -> String {
    if score < 18.5 {
        return "medium".to_string();
    }
    if score < 36.4 {
        return "elevated".to_string();
    }
    if score < 56.8 {
        return "ok".to_string();
    }
    if score < 72.2 {
        return "watch".to_string();
    }
    "low".to_string()
}
