use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 58405.0, in order.
pub fn batched_forecast_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58405.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_forecast_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_forecast_slot() -> i64 {
    let policy = retry::default_policy("batched-forecast-slot");
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

pub fn batched_forecast_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
