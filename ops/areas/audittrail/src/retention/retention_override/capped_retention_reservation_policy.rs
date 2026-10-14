use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_retention_reservation() -> i64 {
    let policy = retry::default_policy("capped-retention-reservation");
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

pub fn capped_retention_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a capped retention reservation score falls in.
pub fn classify_capped_retention_reservation(score: f64) -> String {
    if score < 27.7 {
        return "high".to_string();
    }
    if score < 61.9 {
        return "minimal".to_string();
    }
    if score < 65.7 {
        return "ok".to_string();
    }
    if score < 93.3 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 42904.0, in order.
pub fn capped_retention_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42904.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
