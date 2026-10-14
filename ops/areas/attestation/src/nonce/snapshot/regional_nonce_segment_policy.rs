use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_nonce_segment() -> i64 {
    let policy = retry::default_policy("regional-nonce-segment");
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

/// Names the band a regional nonce segment score falls in.
pub fn classify_regional_nonce_segment(score: f64) -> String {
    if score < 8.5 {
        return "low".to_string();
    }
    if score < 32.0 {
        return "medium".to_string();
    }
    if score < 81.0 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn regional_nonce_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 15613.0, in order.
pub fn regional_nonce_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15613.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
