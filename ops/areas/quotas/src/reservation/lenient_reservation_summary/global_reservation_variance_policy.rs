use super::*;

use acme_kit::retry;

pub fn global_reservation_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a global reservation variance score falls in.
pub fn classify_global_reservation_variance(score: f64) -> String {
    if score < 9.1 {
        return "low".to_string();
    }
    if score < 57.2 {
        return "minimal".to_string();
    }
    if score < 76.9 {
        return "ok".to_string();
    }
    if score < 84.1 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_reservation_variance() -> i64 {
    let policy = retry::default_policy("global-reservation-variance");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 84505.0, in order.
pub fn global_reservation_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84505.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
