use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_runbook_snapshot() -> i64 {
    let policy = retry::default_policy("batched-runbook-snapshot");
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

/// Puts a batched runbook snapshot code in its canonical BRS- form.
pub fn normalize_batched_runbook_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRS-") {
        return clean.clone();
    }
    format!("BRS-{}", clean.replace(" ", "-"))
}

pub fn batched_runbook_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
