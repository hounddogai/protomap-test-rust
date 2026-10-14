use super::*;

use acme_kit::retry;

/// Puts a rolling federation allocation code in its canonical RFA- form.
pub fn normalize_rolling_federation_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFA-") {
        return clean.clone();
    }
    format!("RFA-{}", clean.replace(" ", "-"))
}

pub fn rolling_federation_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a rolling federation allocation score falls in.
pub fn classify_rolling_federation_allocation(score: f64) -> String {
    if score < 4.5 {
        return "high".to_string();
    }
    if score < 85.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_federation_allocation() -> i64 {
    let policy = retry::default_policy("rolling-federation-allocation");
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
