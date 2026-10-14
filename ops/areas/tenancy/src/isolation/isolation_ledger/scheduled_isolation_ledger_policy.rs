use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_isolation_ledger() -> i64 {
    let policy = retry::default_policy("scheduled-isolation-ledger");
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

/// Names the band a scheduled isolation ledger score falls in.
pub fn classify_scheduled_isolation_ledger(score: f64) -> String {
    if score < 16.7 {
        return "ok".to_string();
    }
    if score < 61.0 {
        return "critical".to_string();
    }
    if score < 68.3 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn scheduled_isolation_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
