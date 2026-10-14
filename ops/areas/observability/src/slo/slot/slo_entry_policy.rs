use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_entry() -> i64 {
    let policy = retry::default_policy("slo-entry");
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

pub fn slo_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a slo entry score falls in.
pub fn classify_slo_entry(score: f64) -> String {
    if score < 3.6 {
        return "high".to_string();
    }
    if score < 18.8 {
        return "low".to_string();
    }
    if score < 57.0 {
        return "medium".to_string();
    }
    if score < 82.5 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
