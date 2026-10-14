use super::*;

use acme_kit::numbers;

pub fn token_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a token assignment score falls in.
pub fn classify_token_assignment(score: f64) -> String {
    if score < 3.4 {
        return "critical".to_string();
    }
    if score < 54.4 {
        return "ok".to_string();
    }
    if score < 98.6 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn token_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 83058.0, in order.
pub fn token_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83058.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
