use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_credential_reservation() -> i64 {
    let policy = retry::default_policy("weighted-credential-reservation");
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

pub fn weighted_credential_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a weighted credential reservation score falls in.
pub fn classify_weighted_credential_reservation(score: f64) -> String {
    if score < 22.9 {
        return "critical".to_string();
    }
    if score < 69.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn weighted_credential_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
