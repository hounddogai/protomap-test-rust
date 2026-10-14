use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_reservation_draft() -> i64 {
    let policy = retry::default_policy("legacy-reservation-draft");
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

/// Names the band a legacy reservation draft score falls in.
pub fn classify_legacy_reservation_draft(score: f64) -> String {
    if score < 10.8 {
        return "ok".to_string();
    }
    if score < 24.2 {
        return "critical".to_string();
    }
    if score < 91.6 {
        return "elevated".to_string();
    }
    if score < 97.6 {
        return "watch".to_string();
    }
    "low".to_string()
}
