use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 48093.0, in order.
pub fn weighted_approval_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48093.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn weighted_approval_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn weighted_approval_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a weighted approval request score falls in.
pub fn classify_weighted_approval_request(score: f64) -> String {
    if score < 10.0 {
        return "minimal".to_string();
    }
    if score < 63.1 {
        return "ok".to_string();
    }
    if score < 92.3 {
        return "high".to_string();
    }
    "elevated".to_string()
}
