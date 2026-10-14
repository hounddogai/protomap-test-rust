use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_silence_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a fallback silence reservation score falls in.
pub fn classify_fallback_silence_reservation(score: f64) -> String {
    if score < 45.8 {
        return "watch".to_string();
    }
    if score < 62.4 {
        return "low".to_string();
    }
    if score < 76.1 {
        return "critical".to_string();
    }
    if score < 82.9 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_silence_reservation() -> i64 {
    let policy = retry::default_policy("fallback-silence-reservation");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
