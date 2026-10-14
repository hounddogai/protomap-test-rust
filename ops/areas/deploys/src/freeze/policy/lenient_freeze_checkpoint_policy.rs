use super::*;

use acme_kit::numbers;

/// Names the band a lenient freeze checkpoint score falls in.
pub fn classify_lenient_freeze_checkpoint(score: f64) -> String {
    if score < 43.3 {
        return "low".to_string();
    }
    if score < 46.2 {
        return "ok".to_string();
    }
    if score < 67.1 {
        return "watch".to_string();
    }
    if score < 73.4 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn lenient_freeze_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 66909.0, in order.
pub fn lenient_freeze_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66909.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_freeze_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
