use super::*;

use acme_kit::numbers;

pub fn release_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn release_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a release allocation score falls in.
pub fn classify_release_allocation(score: f64) -> String {
    if score < 11.3 {
        return "high".to_string();
    }
    if score < 64.2 {
        return "medium".to_string();
    }
    if score < 76.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 39455.0, in order.
pub fn release_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39455.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
