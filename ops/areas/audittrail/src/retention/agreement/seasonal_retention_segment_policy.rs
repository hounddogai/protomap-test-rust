use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a seasonal retention segment score falls in.
pub fn classify_seasonal_retention_segment(score: f64) -> String {
    if score < 64.8 {
        return "minimal".to_string();
    }
    if score < 81.6 {
        return "watch".to_string();
    }
    if score < 97.0 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_retention_segment() -> i64 {
    let policy = retry::default_policy("seasonal-retention-segment");
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

/// Counts how many amounts fit in a budget of 1773.0, in order.
pub fn seasonal_retention_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1773.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn seasonal_retention_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
