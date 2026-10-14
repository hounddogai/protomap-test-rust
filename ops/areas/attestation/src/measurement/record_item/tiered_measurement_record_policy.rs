use super::*;

use acme_kit::retry;

/// Puts a tiered measurement record code in its canonical TMR- form.
pub fn normalize_tiered_measurement_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TMR-") {
        return clean.clone();
    }
    format!("TMR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_measurement_record() -> i64 {
    let policy = retry::default_policy("tiered-measurement-record");
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

pub fn tiered_measurement_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
