use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a legacy federation schedule score falls in.
pub fn classify_legacy_federation_schedule(score: f64) -> String {
    if score < 66.0 {
        return "medium".to_string();
    }
    if score < 66.4 {
        return "ok".to_string();
    }
    if score < 95.8 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 2961.0, in order.
pub fn legacy_federation_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2961.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_federation_schedule() -> i64 {
    let policy = retry::default_policy("legacy-federation-schedule");
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

pub fn legacy_federation_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
