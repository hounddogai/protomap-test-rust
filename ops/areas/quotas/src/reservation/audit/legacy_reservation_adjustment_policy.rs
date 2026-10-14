use super::*;

use acme_kit::retry;

/// Names the band a legacy reservation adjustment score falls in.
pub fn classify_legacy_reservation_adjustment(score: f64) -> String {
    if score < 3.7 {
        return "critical".to_string();
    }
    if score < 75.6 {
        return "ok".to_string();
    }
    if score < 79.8 {
        return "elevated".to_string();
    }
    if score < 82.9 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_reservation_adjustment() -> i64 {
    let policy = retry::default_policy("legacy-reservation-adjustment");
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
