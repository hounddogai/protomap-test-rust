use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn peering_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a peering preference score falls in.
pub fn classify_peering_preference(score: f64) -> String {
    if score < 20.9 {
        return "critical".to_string();
    }
    if score < 38.6 {
        return "elevated".to_string();
    }
    if score < 50.7 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_preference() -> i64 {
    let policy = retry::default_policy("peering-preference");
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
