use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a region pin slot score falls in.
pub fn classify_region_pin_slot(score: f64) -> String {
    if score < 6.5 {
        return "elevated".to_string();
    }
    if score < 43.7 {
        return "ok".to_string();
    }
    if score < 89.9 {
        return "watch".to_string();
    }
    if score < 92.1 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_slot() -> i64 {
    let policy = retry::default_policy("region-pin-slot");
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

pub fn region_pin_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 99841.0, in order.
pub fn region_pin_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99841.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
