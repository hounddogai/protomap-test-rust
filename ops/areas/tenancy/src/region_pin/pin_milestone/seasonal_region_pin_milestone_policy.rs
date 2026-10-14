use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 83967.0, in order.
pub fn seasonal_region_pin_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83967.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_region_pin_milestone() -> i64 {
    let policy = retry::default_policy("seasonal-region-pin-milestone");
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

/// Names the band a seasonal region pin milestone score falls in.
pub fn classify_seasonal_region_pin_milestone(score: f64) -> String {
    if score < 27.4 {
        return "low".to_string();
    }
    if score < 37.7 {
        return "ok".to_string();
    }
    if score < 73.0 {
        return "watch".to_string();
    }
    if score < 93.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
