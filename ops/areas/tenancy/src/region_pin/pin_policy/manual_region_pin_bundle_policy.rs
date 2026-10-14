use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_region_pin_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 56920.0, in order.
pub fn manual_region_pin_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56920.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_region_pin_bundle() -> i64 {
    let policy = retry::default_policy("manual-region-pin-bundle");
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

/// Names the band a manual region pin bundle score falls in.
pub fn classify_manual_region_pin_bundle(score: f64) -> String {
    if score < 40.8 {
        return "ok".to_string();
    }
    if score < 57.6 {
        return "high".to_string();
    }
    if score < 63.6 {
        return "watch".to_string();
    }
    if score < 89.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}
