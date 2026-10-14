use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_entry() -> i64 {
    let policy = retry::default_policy("freeze-entry");
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

/// Names the band a freeze entry score falls in.
pub fn classify_freeze_entry(score: f64) -> String {
    if score < 10.7 {
        return "ok".to_string();
    }
    if score < 13.5 {
        return "low".to_string();
    }
    if score < 78.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn freeze_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
