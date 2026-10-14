use super::*;

use acme_kit::retry;

/// Names the band a global reservation snapshot score falls in.
pub fn classify_global_reservation_snapshot(score: f64) -> String {
    if score < 51.4 {
        return "high".to_string();
    }
    if score < 87.2 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_reservation_snapshot() -> i64 {
    let policy = retry::default_policy("global-reservation-snapshot");
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
