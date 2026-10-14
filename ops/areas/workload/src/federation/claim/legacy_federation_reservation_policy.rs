use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a legacy federation reservation score falls in.
pub fn classify_legacy_federation_reservation(score: f64) -> String {
    if score < 27.3 {
        return "medium".to_string();
    }
    if score < 95.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 1375.0, in order.
pub fn legacy_federation_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1375.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_federation_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_federation_reservation() -> i64 {
    let policy = retry::default_policy("legacy-federation-reservation");
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
