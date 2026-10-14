use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_credential_variance() -> i64 {
    let policy = retry::default_policy("weighted-credential-variance");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn weighted_credential_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a weighted credential variance code in its canonical WCV- form.
pub fn normalize_weighted_credential_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WCV-") {
        return clean.clone();
    }
    format!("WCV-{}", clean.replace(" ", "-"))
}
