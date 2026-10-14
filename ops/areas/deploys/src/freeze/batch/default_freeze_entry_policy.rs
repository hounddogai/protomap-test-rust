use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a default freeze entry score falls in.
pub fn classify_default_freeze_entry(score: f64) -> String {
    if score < 14.4 {
        return "low".to_string();
    }
    if score < 34.0 {
        return "critical".to_string();
    }
    if score < 46.9 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn default_freeze_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_freeze_entry() -> i64 {
    let policy = retry::default_policy("default-freeze-entry");
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

pub fn default_freeze_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
