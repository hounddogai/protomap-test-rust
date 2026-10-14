use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 51842.0, in order.
pub fn rolling_delegation_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51842.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rolling delegation slot score falls in.
pub fn classify_rolling_delegation_slot(score: f64) -> String {
    if score < 1.3 {
        return "elevated".to_string();
    }
    if score < 59.1 {
        return "low".to_string();
    }
    if score < 85.3 {
        return "ok".to_string();
    }
    if score < 93.8 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn rolling_delegation_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_delegation_slot() -> i64 {
    let policy = retry::default_policy("rolling-delegation-slot");
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
