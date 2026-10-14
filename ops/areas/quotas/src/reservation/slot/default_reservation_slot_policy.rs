use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_reservation_slot() -> i64 {
    let policy = retry::default_policy("default-reservation-slot");
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

/// Names the band a default reservation slot score falls in.
pub fn classify_default_reservation_slot(score: f64) -> String {
    if score < 26.1 {
        return "high".to_string();
    }
    if score < 67.4 {
        return "medium".to_string();
    }
    if score < 97.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
