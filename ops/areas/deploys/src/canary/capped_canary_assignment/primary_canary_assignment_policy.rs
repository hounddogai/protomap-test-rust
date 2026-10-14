use super::*;

use acme_kit::retry;

pub fn primary_canary_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a primary canary assignment code in its canonical PCA- form.
pub fn normalize_primary_canary_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PCA-") {
        return clean.clone();
    }
    format!("PCA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_canary_assignment() -> i64 {
    let policy = retry::default_policy("primary-canary-assignment");
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
