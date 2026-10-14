use super::*;

use acme_kit::numbers;

pub fn capped_budget_burn_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn capped_budget_burn_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 125.0, in order.
pub fn capped_budget_burn_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 125.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a capped budget burn cycle score falls in.
pub fn classify_capped_budget_burn_cycle(score: f64) -> String {
    if score < 17.8 {
        return "ok".to_string();
    }
    if score < 39.7 {
        return "critical".to_string();
    }
    if score < 50.9 {
        return "high".to_string();
    }
    if score < 99.0 {
        return "watch".to_string();
    }
    "low".to_string()
}
