use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 30745.0, in order.
pub fn batched_region_pin_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30745.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_region_pin_reservation() -> i64 {
    let policy = retry::default_policy("batched-region-pin-reservation");
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

/// Names the band a batched region pin reservation score falls in.
pub fn classify_batched_region_pin_reservation(score: f64) -> String {
    if score < 38.1 {
        return "medium".to_string();
    }
    if score < 66.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
