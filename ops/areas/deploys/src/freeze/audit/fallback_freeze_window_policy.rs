use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_freeze_window() -> i64 {
    let policy = retry::default_policy("fallback-freeze-window");
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

/// Names the band a fallback freeze window score falls in.
pub fn classify_fallback_freeze_window(score: f64) -> String {
    if score < 36.4 {
        return "watch".to_string();
    }
    if score < 47.3 {
        return "elevated".to_string();
    }
    if score < 96.3 {
        return "ok".to_string();
    }
    if score < 99.7 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn fallback_freeze_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
