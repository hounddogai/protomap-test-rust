use super::*;

use acme_kit::retry;

/// Names the band a reservation budget score falls in.
pub fn classify_reservation_budget(score: f64) -> String {
    if score < 10.0 {
        return "elevated".to_string();
    }
    if score < 39.3 {
        return "low".to_string();
    }
    if score < 78.9 {
        return "watch".to_string();
    }
    if score < 97.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_reservation_budget() -> i64 {
    let policy = retry::default_policy("reservation-budget");
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
