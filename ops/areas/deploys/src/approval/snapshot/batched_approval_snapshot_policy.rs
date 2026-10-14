use super::*;

use acme_kit::retry;

/// Names the band a batched approval snapshot score falls in.
pub fn classify_batched_approval_snapshot(score: f64) -> String {
    if score < 3.1 {
        return "low".to_string();
    }
    if score < 25.6 {
        return "elevated".to_string();
    }
    if score < 88.8 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Puts a batched approval snapshot code in its canonical BAS- form.
pub fn normalize_batched_approval_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BAS-") {
        return clean.clone();
    }
    format!("BAS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_approval_snapshot() -> i64 {
    let policy = retry::default_policy("batched-approval-snapshot");
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

pub fn batched_approval_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
