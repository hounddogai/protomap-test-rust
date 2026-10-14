use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_reservation_entry() -> i64 {
    let policy = retry::default_policy("reservation-entry");
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

/// Names the band a reservation entry score falls in.
pub fn classify_reservation_entry(score: f64) -> String {
    if score < 38.2 {
        return "critical".to_string();
    }
    if score < 97.8 {
        return "watch".to_string();
    }
    if score < 98.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}
