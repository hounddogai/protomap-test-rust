use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_milestone() -> i64 {
    let policy = retry::default_policy("allowlist-milestone");
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

/// Puts a allowlist milestone code in its canonical AM- form.
pub fn normalize_allowlist_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AM-") {
        return clean.clone();
    }
    format!("AM-{}", clean.replace(" ", "-"))
}

pub fn allowlist_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a allowlist milestone score falls in.
pub fn classify_allowlist_milestone(score: f64) -> String {
    if score < 27.3 {
        return "watch".to_string();
    }
    if score < 38.7 {
        return "ok".to_string();
    }
    if score < 47.1 {
        return "high".to_string();
    }
    if score < 93.6 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
