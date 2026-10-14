use super::*;

use acme_kit::numbers;

pub fn pending_canary_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending canary bundle score falls in.
pub fn classify_pending_canary_bundle(score: f64) -> String {
    if score < 7.2 {
        return "ok".to_string();
    }
    if score < 18.6 {
        return "low".to_string();
    }
    if score < 98.3 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 94753.0, in order.
pub fn pending_canary_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94753.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
