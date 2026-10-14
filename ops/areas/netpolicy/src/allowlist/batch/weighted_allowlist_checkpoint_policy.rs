use super::*;

use acme_kit::retry;

/// Puts a weighted allowlist checkpoint code in its canonical WAC- form.
pub fn normalize_weighted_allowlist_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WAC-") {
        return clean.clone();
    }
    format!("WAC-{}", clean.replace(" ", "-"))
}

pub fn weighted_allowlist_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_allowlist_checkpoint() -> i64 {
    let policy = retry::default_policy("weighted-allowlist-checkpoint");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
